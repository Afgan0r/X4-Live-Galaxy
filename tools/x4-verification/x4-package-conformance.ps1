[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string]$PackageRoot,
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string]$ContractPath,
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string]$DossierPath,
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string]$CoveragePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:failureCode = 'INTERNAL_VALIDATION_ERROR'
$script:contract = $null
$script:classification = 'non-conformant'
$script:packageId = 'unparsed'
$script:entrypoint = $null
$script:nativeBindingPath = $null
$script:importGraph = @()
$script:dossierDigest = $null
$script:coverageDigest = $null
$script:graphDigest = $null
$script:diagnostics = @()
$script:totalBytes = 0
$script:importCount = 0
$script:visiting = @{}
$script:visited = @{}
$script:sources = @{}

function Fail([string]$Code, [string]$Classification = 'non-conformant', [string]$LogicalPath = '') {
    $script:failureCode = $Code
    $script:classification = $Classification
    if (-not [string]::IsNullOrWhiteSpace($LogicalPath) -and $script:diagnostics.Count -lt $script:contract.bounds.max_diagnostics) {
        $script:diagnostics += $LogicalPath
    }
    throw [System.InvalidOperationException]::new($Code)
}

function Get-Sha256([byte[]]$Bytes) {
    $hash = [System.Security.Cryptography.SHA256]::HashData($Bytes)
    return [Convert]::ToHexString($hash).ToLowerInvariant()
}

function Assert-NoReparsePath([string]$Path, [string]$FailureCode) {
    $full = [IO.Path]::GetFullPath($Path)
    $root = [IO.Path]::GetPathRoot($full)
    $current = $root
    foreach ($segment in @($full.Substring($root.Length) -split '[\\/]+' | Where-Object { $_ -ne '' })) {
        $current = Join-Path $current $segment
        $item = Get-Item -LiteralPath $current -Force -ErrorAction SilentlyContinue
        if ($null -eq $item) { Fail $FailureCode }
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or $null -ne $item.LinkType) {
            Fail 'REPARSE_POINT_ESCAPE' 'non-conformant'
        }
    }
}

function Read-BoundedBytes([string]$Path, [int]$Maximum, [string]$FailureCode) {
    Assert-NoReparsePath $Path $FailureCode
    $before = Get-Item -LiteralPath $Path -Force
    if ($before.PSIsContainer) { Fail $FailureCode }
    $bytes = [System.IO.File]::ReadAllBytes($before.FullName)
    Assert-NoReparsePath $Path $FailureCode
    $after = Get-Item -LiteralPath $Path -Force
    if ($after.FullName -ne $before.FullName -or $after.Length -ne $before.Length -or
        $after.LastWriteTimeUtc -ne $before.LastWriteTimeUtc) { Fail 'PATH_IDENTITY_CHANGED' }
    if ($bytes.Length -gt $Maximum) { Fail 'FILE_BYTES_EXCEEDED' }
    return $bytes
}

function Read-JsonContract([string]$Path, [string]$Schema) {
    $bytes = Read-BoundedBytes $Path 131072 'MISSING_INPUT'
    try { $value = [Text.Encoding]::UTF8.GetString($bytes) | ConvertFrom-Json -Depth 64 }
    catch { Fail 'INVALID_JSON' }
    if ($value.schema_version -ne $Schema) { Fail 'UNSUPPORTED_SCHEMA' }
    return [pscustomobject]@{ Value = $value; Bytes = $bytes }
}

function Read-Xml([string]$LogicalPath, [string]$MissingCode) {
    $resolved = Resolve-PackagePath $LogicalPath $MissingCode
    $bytes = Read-BoundedBytes $resolved.FullPath $script:contract.bounds.max_file_bytes $MissingCode
    try {
        $document = [System.Xml.XmlDocument]::new()
        $document.XmlResolver = $null
        $document.LoadXml([Text.Encoding]::UTF8.GetString($bytes))
        return $document
    }
    catch { Fail 'INVALID_XML' 'non-conformant' $LogicalPath }
}

function Resolve-PackagePath([string]$LogicalPath, [string]$MissingCode = 'UNRESOLVED_IMPORT') {
    if ([string]::IsNullOrWhiteSpace($LogicalPath) -or [IO.Path]::IsPathRooted($LogicalPath) -or $LogicalPath -match '(^|[\\/])\.\.([\\/]|$)') {
        Fail 'ROOT_ESCAPE' 'non-conformant' $LogicalPath
    }
    $normalized = ($LogicalPath -replace '\\', '/').TrimStart('/')
    $current = $script:packageRoot
    $rootItem = Get-Item -LiteralPath $current -Force
    if (($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or $null -ne $rootItem.LinkType) {
        Fail 'PACKAGE_ROOT_REPARSE_POINT' 'non-conformant'
    }
    foreach ($segment in @($normalized -split '/')) {
        if ([string]::IsNullOrWhiteSpace($segment) -or $segment -eq '.') { Fail 'ROOT_ESCAPE' 'non-conformant' $normalized }
        $match = @(Get-ChildItem -LiteralPath $current -Force -ErrorAction SilentlyContinue | Where-Object { $_.Name -ceq $segment })
        if ($match.Count -ne 1) { Fail $MissingCode 'non-conformant' $normalized }
        if (($match[0].Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or $null -ne $match[0].LinkType) {
            Fail 'REPARSE_POINT_ESCAPE' 'non-conformant' $normalized
        }
        $current = $match[0].FullName
    }
    $full = [IO.Path]::GetFullPath($current)
    $prefix = $script:packageRoot.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    if (-not $full.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) { Fail 'ROOT_ESCAPE' 'non-conformant' $normalized }
    return [pscustomobject]@{ FullPath = $full; LogicalPath = $normalized }
}

function Get-LuaTokens([string]$Source) {
    $tokens = [Collections.Generic.List[object]]::new()
    $index = 0
    while ($index -lt $Source.Length) {
        $character = $Source[$index]
        if ([char]::IsWhiteSpace($character)) { $index++; continue }
        if ($character -eq '-' -and $index + 1 -lt $Source.Length -and $Source[$index + 1] -eq '-') {
            if ($index + 3 -lt $Source.Length -and $Source.Substring($index, 4) -eq '--[[') {
                $end = $Source.IndexOf(']]', $index + 4, [StringComparison]::Ordinal)
                if ($end -lt 0) { Fail 'INVALID_LUA_SOURCE' }
                $index = $end + 2
            }
            else {
                $end = $Source.IndexOf("`n", $index + 2, [StringComparison]::Ordinal)
                $index = if ($end -lt 0) { $Source.Length } else { $end + 1 }
            }
            continue
        }
        if ($character -eq '[' -and $index + 1 -lt $Source.Length -and $Source[$index + 1] -eq '[') {
            $end = $Source.IndexOf(']]', $index + 2, [StringComparison]::Ordinal)
            if ($end -lt 0) { Fail 'INVALID_LUA_SOURCE' }
            $tokens.Add([pscustomobject]@{ Kind = 'string'; Value = $Source.Substring($index + 2, $end - $index - 2) })
            $index = $end + 2
            continue
        }
        if ($character -eq '"' -or $character -eq "'") {
            $quote = $character
            $builder = [Text.StringBuilder]::new()
            $index++
            $closed = $false
            while ($index -lt $Source.Length) {
                $next = $Source[$index]
                if ($next -eq '\\') {
                    if ($index + 1 -ge $Source.Length) { Fail 'INVALID_LUA_SOURCE' }
                    [void]$builder.Append($Source[$index + 1]); $index += 2; continue
                }
                if ($next -eq $quote) { $closed = $true; $index++; break }
                [void]$builder.Append($next); $index++
            }
            if (-not $closed) { Fail 'INVALID_LUA_SOURCE' }
            $tokens.Add([pscustomobject]@{ Kind = 'string'; Value = $builder.ToString() })
            continue
        }
        if ([char]::IsLetter($character) -or $character -eq '_') {
            $start = $index; $index++
            while ($index -lt $Source.Length -and ([char]::IsLetterOrDigit($Source[$index]) -or $Source[$index] -eq '_')) { $index++ }
            $tokens.Add([pscustomobject]@{ Kind = 'identifier'; Value = $Source.Substring($start, $index - $start) })
            continue
        }
        if ($character -eq '.' -and $index + 1 -lt $Source.Length -and $Source[$index + 1] -eq '.') {
            $tokens.Add([pscustomobject]@{ Kind = 'symbol'; Value = '..' }); $index += 2; continue
        }
        $tokens.Add([pscustomobject]@{ Kind = 'symbol'; Value = [string]$character }); $index++
    }
    return @($tokens)
}

function Resolve-StaticExpression($Tokens, [int]$Start, [int]$End, $Constants) {
    $parts = [Collections.Generic.List[string]]::new()
    $expectValue = $true
    for ($i = $Start; $i -lt $End; $i++) {
        $token = $Tokens[$i]
        if ($expectValue) {
            if ($token.Kind -eq 'string') { $parts.Add($token.Value) }
            elseif ($token.Kind -eq 'identifier' -and $Constants.ContainsKey($token.Value)) { $parts.Add($Constants[$token.Value]) }
            else { return $null }
        }
        elseif ($token.Value -ne '..') { return $null }
        $expectValue = -not $expectValue
    }
    if ($expectValue) { return $null }
    return ($parts -join '')
}

function Get-Imports([string]$Source) {
    $tokens = @(Get-LuaTokens $Source)
    $imports = [Collections.Generic.List[string]]::new()
    $constants = @{}
    $helpers = @{}
    $helperDeclarationIndexes = @{}
    $helperRequireIndexes = @{}
    for ($i = 0; $i + 3 -lt $tokens.Count; $i++) {
        if ($tokens[$i].Value -eq 'local' -and $tokens[$i + 1].Kind -eq 'identifier' -and $tokens[$i + 2].Value -eq '=') {
            $end = $i + 3
            while ($end -lt $tokens.Count -and $tokens[$end].Value -notin @('local', 'return', 'if', 'function', 'end')) { $end++ }
            $resolved = Resolve-StaticExpression $tokens ($i + 3) $end $constants
            if ($null -ne $resolved) { $constants[$tokens[$i + 1].Value] = $resolved }
        }
    }
    for ($i = 0; $i + 8 -lt $tokens.Count; $i++) {
        if ($tokens[$i].Value -ne 'local' -or $tokens[$i + 1].Value -ne 'function' -or
            $tokens[$i + 2].Kind -ne 'identifier' -or $tokens[$i + 3].Value -ne '(' -or
            $tokens[$i + 4].Kind -ne 'identifier' -or $tokens[$i + 5].Value -ne ')') { continue }
        $helperName = $tokens[$i + 2].Value
        $parameterName = $tokens[$i + 4].Value
        for ($j = $i + 6; $j + 3 -lt $tokens.Count -and $tokens[$j].Value -ne 'end'; $j++) {
            if ($tokens[$j].Value -ne 'require' -or $tokens[$j + 1].Value -ne '(') { continue }
            $end = $j + 2
            while ($end -lt $tokens.Count -and $tokens[$end].Value -ne ')') { $end++ }
            if ($end -ge $tokens.Count) { Fail 'INVALID_LUA_SOURCE' }
            $parameterIndex = -1
            for ($k = $j + 2; $k -lt $end; $k++) {
                if ($tokens[$k].Value -eq $parameterName) {
                    if ($parameterIndex -ge 0) { Fail 'DYNAMIC_REQUIRE' 'local-only' }
                    $parameterIndex = $k
                }
            }
            if ($parameterIndex -lt 0) { continue }
            $prefixEnd = $parameterIndex
            if ($parameterIndex -gt $j + 2 -and $tokens[$parameterIndex - 1].Value -eq '..') { $prefixEnd-- }
            $suffixStart = $parameterIndex + 1
            if ($suffixStart -lt $end -and $tokens[$suffixStart].Value -eq '..') { $suffixStart++ }
            $prefix = if ($prefixEnd -eq $j + 2) { '' } else { Resolve-StaticExpression $tokens ($j + 2) $prefixEnd $constants }
            $suffix = if ($suffixStart -eq $end) { '' } else { Resolve-StaticExpression $tokens $suffixStart $end $constants }
            if ($null -eq $prefix -or $null -eq $suffix) { Fail 'DYNAMIC_REQUIRE' 'local-only' }
            $helpers[$helperName] = [pscustomobject]@{ Prefix = $prefix; Suffix = $suffix }
            $helperDeclarationIndexes[$i + 2] = $true
            $helperRequireIndexes[$j] = $true
        }
    }
    for ($i = 0; $i -lt $tokens.Count; $i++) {
        if ($tokens[$i].Kind -eq 'identifier' -and $helpers.ContainsKey($tokens[$i].Value)) {
            if ($helperDeclarationIndexes.ContainsKey($i)) { continue }
            if ($i + 1 -ge $tokens.Count -or $tokens[$i + 1].Value -ne '(') {
                Fail 'REQUIRE_HELPER_ALIAS_UNSUPPORTED' 'local-only'
            }
            $end = $i + 2
            $depth = 1
            while ($end -lt $tokens.Count -and $depth -gt 0) {
                if ($tokens[$end].Value -eq '(') { $depth++ }
                elseif ($tokens[$end].Value -eq ')') { $depth-- }
                $end++
            }
            if ($depth -ne 0) { Fail 'INVALID_LUA_SOURCE' }
            $argument = Resolve-StaticExpression $tokens ($i + 2) ($end - 1) $constants
            if ($null -eq $argument) { Fail 'DYNAMIC_REQUIRE' 'local-only' }
            $helper = $helpers[$tokens[$i].Value]
            $imports.Add($helper.Prefix + $argument + $helper.Suffix)
            $i = $end - 1
            continue
        }
        if ($tokens[$i].Value -eq 'require') {
            if ($helperRequireIndexes.ContainsKey($i)) { continue }
            if ($i -gt 0 -and $tokens[$i - 1].Value -eq 'local') { continue }
            $start = $i + 1
            if ($start -lt $tokens.Count -and $tokens[$start].Value -eq '(') {
                $end = $start + 1; $depth = 1
                while ($end -lt $tokens.Count -and $depth -gt 0) {
                    if ($tokens[$end].Value -eq '(') { $depth++ }
                    elseif ($tokens[$end].Value -eq ')') { $depth-- }
                    $end++
                }
                if ($depth -ne 0) { Fail 'INVALID_LUA_SOURCE' }
                $module = Resolve-StaticExpression $tokens ($start + 1) ($end - 1) $constants
                if ($null -eq $module) { Fail 'DYNAMIC_REQUIRE' 'local-only' }
                $imports.Add($module); $i = $end - 1
            }
            elseif ($i -gt 1 -and $tokens[$i - 1].Value -eq '(' -and $tokens[$i - 2].Value -eq 'pcall') { continue }
            else { Fail 'REQUIRE_ALIAS_UNSUPPORTED' 'local-only' }
        }
        elseif ($tokens[$i].Value -eq 'pcall' -and $i + 4 -lt $tokens.Count -and $tokens[$i + 1].Value -eq '(' -and $tokens[$i + 2].Value -eq 'require' -and $tokens[$i + 3].Value -eq ',') {
            $end = $i + 4
            while ($end -lt $tokens.Count -and $tokens[$end].Value -ne ')') { $end++ }
            if ($end -ge $tokens.Count) { Fail 'INVALID_LUA_SOURCE' }
            $module = Resolve-StaticExpression $tokens ($i + 4) $end $constants
            if ($null -eq $module) { Fail 'DYNAMIC_REQUIRE' 'local-only' }
            $imports.Add($module); $i = $end
        }
    }
    return @($imports)
}

function Resolve-InternalModule([string]$Module) {
    foreach ($prefix in @($script:contract.test_only_prefixes)) {
        if ($Module.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) { Fail 'TEST_ONLY_DEPENDENCY' 'local-only' $Module }
    }
    if ($Module -match '(^|[\\/])\.\.([\\/]|$)') { Fail 'ROOT_ESCAPE' 'non-conformant' $Module }
    if ($Module.StartsWith($script:contract.internal_module_prefix, [StringComparison]::Ordinal)) {
        $suffix = $Module.Substring($script:contract.internal_module_prefix.Length)
        return "lua/$suffix.lua"
    }
    if (@($script:contract.external_modules) -ccontains $Module) { return $null }
    Fail 'BARE_PRODUCTION_IMPORT' 'local-only' $Module
}

function Visit-Module([string]$LogicalPath, [int]$Depth) {
    if ($Depth -gt $script:contract.bounds.max_depth) { Fail 'GRAPH_DEPTH_EXCEEDED' 'non-conformant' $LogicalPath }
    if ($script:visiting.ContainsKey($LogicalPath)) { Fail 'IMPORT_CYCLE' 'non-conformant' $LogicalPath }
    if ($script:visited.ContainsKey($LogicalPath)) { return }
    if ($script:visited.Count -ge $script:contract.bounds.max_files) { Fail 'GRAPH_SIZE_EXCEEDED' 'non-conformant' $LogicalPath }

    $resolved = Resolve-PackagePath $LogicalPath 'UNRESOLVED_IMPORT'
    $bytes = Read-BoundedBytes $resolved.FullPath $script:contract.bounds.max_file_bytes 'UNRESOLVED_IMPORT'
    $script:totalBytes += $bytes.Length
    if ($script:totalBytes -gt $script:contract.bounds.max_total_bytes) { Fail 'TOTAL_BYTES_EXCEEDED' 'non-conformant' $LogicalPath }
    $source = [Text.Encoding]::UTF8.GetString($bytes)
    $script:visiting[$LogicalPath] = $true
    $script:sources[$LogicalPath] = [pscustomobject]@{ Bytes = $bytes; Source = $source }

    $imports = @(Get-Imports $source)
    if (@($imports | Sort-Object -Unique).Count -ne $imports.Count) { Fail 'DUPLICATE_IMPORT' 'non-conformant' $LogicalPath }
    $script:importCount += $imports.Count
    if ($script:importCount -gt $script:contract.bounds.max_imports) { Fail 'IMPORT_COUNT_EXCEEDED' 'non-conformant' $LogicalPath }

    foreach ($module in $imports) {
        $child = Resolve-InternalModule $module
        if ($null -ne $child) { Visit-Module $child ($Depth + 1) }
    }
    $script:visiting.Remove($LogicalPath)
    $script:visited[$LogicalPath] = $true
}

function Test-AdmissionContext($Dossier, $Coverage) {
    foreach ($dimensionId in @($script:contract.admission_dimensions)) {
        $matches = @($Dossier.dimensions | Where-Object { $_.id -eq $dimensionId -and $_.status -eq 'EVIDENCED' })
        if ($matches.Count -ne 1) { Fail 'ADMISSION_CONTEXT_INVALID' }
    }
    foreach ($failureClassId in @($script:contract.admission_failure_classes)) {
        $matches = @($Coverage.rows | Where-Object { $_.failure_class_id -eq $failureClassId -and $_.status -eq 'covered' })
        if ($matches.Count -ne 1) { Fail 'ADMISSION_CONTEXT_INVALID' }
    }
}

function Write-Result([string]$Verdict, [string[]]$ReasonCodes) {
    $result = [ordered]@{
        schema_version = 'x4-package-conformance-result.v1'
        verdict = $Verdict
        classification = $script:classification
        evidence_level = 'packaged-static'
        package_id = $script:packageId
        entrypoint = $script:entrypoint
        import_graph = @($script:importGraph)
        native_binding_path = $script:nativeBindingPath
        reason_codes = @($ReasonCodes | Select-Object -First $script:contract.bounds.max_diagnostics)
        diagnostics = @($script:diagnostics | Select-Object -First $script:contract.bounds.max_diagnostics)
        dossier_digest = $script:dossierDigest
        coverage_digest = $script:coverageDigest
        graph_digest = $script:graphDigest
    }
    Write-Output ($result | ConvertTo-Json -Compress -Depth 16)
}

try {
    $script:packageRoot = (Resolve-Path -LiteralPath $PackageRoot -ErrorAction Stop).Path
    if (-not (Test-Path -LiteralPath $script:packageRoot -PathType Container)) { Fail 'MISSING_PACKAGE_ROOT' }
    $contractRead = Read-JsonContract $ContractPath 'x4-package-conformance.v1'
    $script:contract = $contractRead.Value
    $script:packageId = $script:contract.package_id
    $dossierRead = Read-JsonContract $DossierPath 'x4-integration-dossier.v1'
    $coverageRead = Read-JsonContract $CoveragePath 'x4-known-failure-coverage.v1'
    $script:dossierDigest = Get-Sha256 $dossierRead.Bytes
    $script:coverageDigest = Get-Sha256 $coverageRead.Bytes
    Test-AdmissionContext $dossierRead.Value $coverageRead.Value

    $content = Read-Xml 'content.xml' 'MISSING_CONTENT_MANIFEST'
    $ui = Read-Xml 'ui.xml' 'MISSING_UI_REGISTRATION'
    $contentDependencies = @($content.DocumentElement.SelectNodes('./dependency'))
    if ($content.DocumentElement.GetAttribute('id') -cne $script:contract.package_id -or
        @($contentDependencies | Where-Object { $_.GetAttribute('id') -ceq $script:contract.required_content_dependency }).Count -ne 1) {
        Fail 'INVALID_PACKAGE_IDENTITY'
    }
    $environment = @($ui.DocumentElement.SelectNodes('./environment') | Where-Object {
        $_.GetAttribute('type') -ceq $script:contract.required_environment
    })
    if ($environment.Count -ne 1) { Fail 'MISSING_REGISTRATION' }
    $files = @($environment[0].SelectNodes('./file'))
    if ($files.Count -ne 1) { Fail 'MISSING_REGISTRATION' }
    $uiDependencies = @($environment[0].SelectNodes('./dependency'))
    if (@($uiDependencies | Where-Object { $_.GetAttribute('name') -ceq $script:contract.required_ui_dependency }).Count -ne 1) {
        Fail 'MISSING_REGISTRATION'
    }
    $script:entrypoint = $files[0].GetAttribute('name')
    if ($script:entrypoint -cne $script:contract.required_entrypoint) { Fail 'WRONG_ENTRYPOINT' 'non-conformant' $script:entrypoint }

    Visit-Module $script:entrypoint 0
    $script:importGraph = @($script:visited.Keys | Sort-Object)
    $bindingPaths = @()
    $alternateBinding = $false
    foreach ($logicalPath in $script:importGraph) {
        $source = $script:sources[$logicalPath].Source
        $imports = @(Get-Imports $source)
        $hasFfi = $imports -ccontains $script:contract.native_binding.module
        $hasBinding = $source -match '(?m)^\s*local\s+C\s*=\s*ffi\.C\s*$'
        if ($hasFfi -and $hasBinding) { $bindingPaths += $logicalPath }
        elseif ($source -match '(?:globals|_G)\.C|require\s*\([^\r\n]*binding') { $alternateBinding = $true }
    }
    if ($bindingPaths.Count -gt 1 -or ($bindingPaths.Count -eq 0 -and $alternateBinding)) {
        Fail 'ALTERNATE_BINDING_SOURCE' 'local-only'
    }
    if ($bindingPaths.Count -ne 1) { Fail 'NATIVE_BINDING_NOT_FOUND' }
    $script:nativeBindingPath = $bindingPaths[0]

    $digestLines = foreach ($logicalPath in $script:importGraph) {
        "$logicalPath=$((Get-Sha256 $script:sources[$logicalPath].Bytes))"
    }
    $script:graphDigest = Get-Sha256 ([Text.Encoding]::UTF8.GetBytes(($digestLines -join "`n")))
    $repositoryRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
    $canonicalPackageRoot = [IO.Path]::GetFullPath((Join-Path $repositoryRoot 'extensions/live_galaxy'))
    $script:classification = if ($script:packageRoot -eq $canonicalPackageRoot) { 'production-faithful' } else { 'local-only' }
    Write-Result 'conformant' @('CONFORMANT')
    exit 0
}
catch {
    if ($null -eq $script:contract) {
        $script:contract = [pscustomobject]@{ bounds = [pscustomobject]@{ max_diagnostics = 16 } }
    }
    Write-Result 'non-conformant' @($script:failureCode)
    exit 1
}
