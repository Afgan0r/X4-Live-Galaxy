[CmdletBinding()]
param(
    [ValidateNotNullOrEmpty()]
    [string]$PackageRoot = (Split-Path -Parent $PSScriptRoot)
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Product expectations are independent of the package being checked.
# These finite traversal safeguards are tool limits, not runtime policy.
$script:contract = @{
    package_id = 'live_galaxy'
    required_content_dependency = 'ws_2042901274'
    required_ui_dependency = 'sn_mod_support_apis'
    required_environment = 'menus'
    required_entrypoint = 'lua/live_galaxy_runtime.lua'
    internal_module_prefix = 'live_galaxy/lua/'
    external_modules = @('ffi', 'extensions.sn_mod_support_apis.ui.named_pipes.Interface')
    test_only_prefixes = @('tests/', 'test/', 'spec/')
    bounds = @{
        max_file_bytes = 32768; max_total_bytes = 131072
        max_files = 8; max_imports = 16; max_depth = 4; max_diagnostics = 16
    }
}
$script:failureCode = 'PACKAGE_READ_FAILED'
$script:diagnostics = @()
$script:totalBytes = 0
$script:importCount = 0
$script:visiting = @{}
$script:visited = @{}
$script:sources = @{}

function Fail([string]$Code, [string]$LogicalPath = '') {
    $script:failureCode = $Code
    if ($LogicalPath -and $script:diagnostics.Count -lt $script:contract.bounds.max_diagnostics) {
        $script:diagnostics += $LogicalPath
    }
    throw [InvalidOperationException]::new($Code)
}

function Assert-NoReparsePath([string]$Path) {
    $full = [IO.Path]::GetFullPath($Path)
    $root = [IO.Path]::GetPathRoot($full)
    $current = $root
    foreach ($segment in @($full.Substring($root.Length) -split '[\\/]+' | Where-Object { $_ -ne '' })) {
        $current = Join-Path $current $segment
        $item = Get-Item -LiteralPath $current -Force -ErrorAction SilentlyContinue
        if ($null -eq $item) { Fail 'MISSING_PACKAGE_ROOT' }
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            Fail 'REPARSE_POINT_ESCAPE'
        }
    }
}

function Read-BoundedBytes([string]$Path, [int]$Maximum, [string]$MissingCode, [string]$LogicalPath) {
    $item = Get-Item -LiteralPath $Path -Force -ErrorAction SilentlyContinue
    if ($null -eq $item -or $item.PSIsContainer) { Fail $MissingCode $LogicalPath }
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        Fail 'REPARSE_POINT_ESCAPE' $LogicalPath
    }
    if ($item.Length -gt $Maximum) { Fail 'FILE_BYTES_EXCEEDED' $LogicalPath }
    try { return ,([IO.File]::ReadAllBytes($Path)) }
    catch { Fail 'PACKAGE_READ_FAILED' $LogicalPath }
}

function Read-Xml([string]$LogicalPath, [string]$MissingCode) {
    $resolved = Resolve-PackagePath $LogicalPath $MissingCode
    $bytes = Read-BoundedBytes $resolved.FullPath $script:contract.bounds.max_file_bytes $MissingCode $LogicalPath
    try {
        $document = [System.Xml.XmlDocument]::new()
        $document.XmlResolver = $null
        $document.LoadXml([Text.Encoding]::UTF8.GetString($bytes))
        return $document
    }
    catch { Fail 'INVALID_XML' $LogicalPath }
}

function Resolve-PackagePath([string]$LogicalPath, [string]$MissingCode = 'UNRESOLVED_IMPORT') {
    if ([string]::IsNullOrWhiteSpace($LogicalPath) -or [IO.Path]::IsPathRooted($LogicalPath) -or $LogicalPath -match '(^|[\\/])\.\.([\\/]|$)') {
        Fail 'ROOT_ESCAPE' $LogicalPath
    }
    $normalized = ($LogicalPath -replace '\\', '/').TrimStart('/')
    $current = $script:packageRoot
    $rootItem = Get-Item -LiteralPath $current -Force
    if (($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or $null -ne $rootItem.LinkType) {
        Fail 'PACKAGE_ROOT_REPARSE_POINT'
    }
    foreach ($segment in @($normalized -split '/')) {
        if ([string]::IsNullOrWhiteSpace($segment) -or $segment -eq '.') { Fail 'ROOT_ESCAPE' $normalized }
        $match = @(Get-ChildItem -LiteralPath $current -Force -ErrorAction SilentlyContinue | Where-Object { $_.Name -ceq $segment })
        if ($match.Count -ne 1) { Fail $MissingCode $normalized }
        if (($match[0].Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or $null -ne $match[0].LinkType) {
            Fail 'REPARSE_POINT_ESCAPE' $normalized
        }
        $current = $match[0].FullName
    }
    $full = [IO.Path]::GetFullPath($current)
    $prefix = $script:packageRoot.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    if (-not $full.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) { Fail 'ROOT_ESCAPE' $normalized }
    return [pscustomobject]@{ FullPath = $full; LogicalPath = $normalized }
}

function Get-LuaLongBracket([string]$Source, [int]$Index) {
    if ($Index -ge $Source.Length -or $Source[$Index] -ne '[') { return $null }
    $cursor = $Index + 1
    while ($cursor -lt $Source.Length -and $Source[$cursor] -eq '=') { $cursor++ }
    if ($cursor -ge $Source.Length -or $Source[$cursor] -ne '[') { return $null }
    $equals = $Source.Substring($Index + 1, $cursor - $Index - 1)
    return [pscustomobject]@{
        ContentStart = $cursor + 1
        Close = "]$equals]"
    }
}

function Get-LuaTokens([string]$Source) {
    $tokens = [Collections.Generic.List[object]]::new()
    $index = 0
    while ($index -lt $Source.Length) {
        $character = $Source[$index]
        if ([char]::IsWhiteSpace($character)) { $index++; continue }
        if ($character -eq '-' -and $index + 1 -lt $Source.Length -and $Source[$index + 1] -eq '-') {
            $longComment = Get-LuaLongBracket $Source ($index + 2)
            if ($null -ne $longComment) {
                $end = $Source.IndexOf($longComment.Close, $longComment.ContentStart, [StringComparison]::Ordinal)
                if ($end -lt 0) { Fail 'INVALID_LUA_SOURCE' }
                $index = $end + $longComment.Close.Length
            }
            else {
                $end = $Source.IndexOf("`n", $index + 2, [StringComparison]::Ordinal)
                $index = if ($end -lt 0) { $Source.Length } else { $end + 1 }
            }
            continue
        }
        $longString = Get-LuaLongBracket $Source $index
        if ($null -ne $longString) {
            $end = $Source.IndexOf($longString.Close, $longString.ContentStart, [StringComparison]::Ordinal)
            if ($end -lt 0) { Fail 'INVALID_LUA_SOURCE' }
            $tokens.Add([pscustomobject]@{
                Kind = 'string'
                Value = $Source.Substring($longString.ContentStart, $end - $longString.ContentStart)
            })
            $index = $end + $longString.Close.Length
            continue
        }
        if ($character -eq '"' -or $character -eq "'") {
            $quote = $character
            $builder = [Text.StringBuilder]::new()
            $index++
            $closed = $false
            while ($index -lt $Source.Length) {
                $next = $Source[$index]
                if ($next -eq '\') {
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

function Get-ExecutableFfiCAccesses([object[]]$Tokens) {
    $accessCount = 0
    $approvedCount = 0
    for ($index = 0; $index + 2 -lt $Tokens.Count; $index++) {
        if ($Tokens[$index].Kind -cne 'identifier' -or
            $Tokens[$index].Value -cne 'ffi') { continue }
        $isMemberAccess = $Tokens[$index + 1].Value -ceq '.' -and
            $Tokens[$index + 2].Kind -ceq 'identifier' -and
            $Tokens[$index + 2].Value -ceq 'C'
        $isIndexedAccess = $index + 3 -lt $Tokens.Count -and
            $Tokens[$index + 1].Value -ceq '[' -and
            $Tokens[$index + 2].Kind -ceq 'string' -and
            $Tokens[$index + 2].Value -ceq 'C' -and
            $Tokens[$index + 3].Value -ceq ']'
        if (-not $isMemberAccess -and -not $isIndexedAccess) { continue }

        $accessCount += 1
        if ($isMemberAccess -and $index -ge 3 -and
            $Tokens[$index - 3].Value -ceq 'local' -and
            $Tokens[$index - 2].Kind -ceq 'identifier' -and
            $Tokens[$index - 1].Value -ceq '=') {
            $approvedCount += 1
        }
    }
    return [pscustomobject]@{
        AccessCount = $accessCount
        ApprovedCount = $approvedCount
    }
}

function Test-ExecutableAlternateBinding([object[]]$Tokens) {
    for ($index = 0; $index + 2 -lt $Tokens.Count; $index++) {
        $values = @($Tokens[$index..($index + 2)] | ForEach-Object Value)
        if (($values[0] -cin @('globals', '_G')) -and
            ($values -join '|') -ceq "$($values[0])|.|C") { return $true }
    }
    return $false
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
                    if ($parameterIndex -ge 0) { Fail 'DYNAMIC_REQUIRE' }
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
            if ($null -eq $prefix -or $null -eq $suffix) { Fail 'DYNAMIC_REQUIRE' }
            $helpers[$helperName] = [pscustomobject]@{ Prefix = $prefix; Suffix = $suffix }
            $helperDeclarationIndexes[$i + 2] = $true
            $helperRequireIndexes[$j] = $true
        }
    }
    for ($i = 0; $i -lt $tokens.Count; $i++) {
        if ($tokens[$i].Kind -eq 'identifier' -and $helpers.ContainsKey($tokens[$i].Value)) {
            if ($helperDeclarationIndexes.ContainsKey($i)) { continue }
            if ($i + 1 -ge $tokens.Count -or $tokens[$i + 1].Value -ne '(') {
                Fail 'REQUIRE_HELPER_ALIAS_UNSUPPORTED'
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
            if ($null -eq $argument) { Fail 'DYNAMIC_REQUIRE' }
            $helper = $helpers[$tokens[$i].Value]
            $imports.Add($helper.Prefix + $argument + $helper.Suffix)
            $i = $end - 1
            continue
        }
        if ($tokens[$i].Kind -eq 'identifier' -and $tokens[$i].Value -eq 'require') {
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
                if ($null -eq $module) { Fail 'DYNAMIC_REQUIRE' }
                $imports.Add($module); $i = $end - 1
            }
            elseif ($i -gt 1 -and $tokens[$i - 1].Value -eq '(' -and $tokens[$i - 2].Value -eq 'pcall') { continue }
            else { Fail 'REQUIRE_ALIAS_UNSUPPORTED' }
        }
        elseif ($tokens[$i].Kind -eq 'identifier' -and $tokens[$i].Value -eq 'pcall' -and $i + 4 -lt $tokens.Count -and $tokens[$i + 1].Value -eq '(' -and $tokens[$i + 2].Value -eq 'require' -and $tokens[$i + 3].Value -eq ',') {
            $end = $i + 4
            while ($end -lt $tokens.Count -and $tokens[$end].Value -ne ')') { $end++ }
            if ($end -ge $tokens.Count) { Fail 'INVALID_LUA_SOURCE' }
            $module = Resolve-StaticExpression $tokens ($i + 4) $end $constants
            if ($null -eq $module) { Fail 'DYNAMIC_REQUIRE' }
            $imports.Add($module); $i = $end
        }
    }
    return @($imports)
}

function Resolve-InternalModule([string]$Module) {
    foreach ($prefix in @($script:contract.test_only_prefixes)) {
        if ($Module.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) { Fail 'TEST_ONLY_DEPENDENCY' $Module }
    }
    if ($Module -match '(^|[\\/])\.\.([\\/]|$)') { Fail 'ROOT_ESCAPE' $Module }
    if ($Module.StartsWith($script:contract.internal_module_prefix, [StringComparison]::Ordinal)) {
        $suffix = $Module.Substring($script:contract.internal_module_prefix.Length)
        return "lua/$suffix.lua"
    }
    if (@($script:contract.external_modules) -ccontains $Module) { return $null }
    Fail 'BARE_PRODUCTION_IMPORT' $Module
}

function Visit-Module([string]$LogicalPath, [int]$Depth) {
    if ($Depth -gt $script:contract.bounds.max_depth) { Fail 'GRAPH_DEPTH_EXCEEDED' $LogicalPath }
    if ($script:visiting.ContainsKey($LogicalPath)) { Fail 'IMPORT_CYCLE' $LogicalPath }
    if ($script:visited.ContainsKey($LogicalPath)) { return }
    if ($script:sources.Count -ge $script:contract.bounds.max_files) { Fail 'GRAPH_SIZE_EXCEEDED' $LogicalPath }

    $resolved = Resolve-PackagePath $LogicalPath 'UNRESOLVED_IMPORT'
    $bytes = Read-BoundedBytes $resolved.FullPath $script:contract.bounds.max_file_bytes 'UNRESOLVED_IMPORT' $LogicalPath
    $script:totalBytes += $bytes.Length
    if ($script:totalBytes -gt $script:contract.bounds.max_total_bytes) { Fail 'TOTAL_BYTES_EXCEEDED' $LogicalPath }
    $source = [Text.Encoding]::UTF8.GetString($bytes)
    $script:visiting[$LogicalPath] = $true
    $script:sources[$LogicalPath] = $source

    $imports = @(Get-Imports $source)
    if (@($imports | Sort-Object -Unique).Count -ne $imports.Count) { Fail 'DUPLICATE_IMPORT' $LogicalPath }
    $script:importCount += $imports.Count
    if ($script:importCount -gt $script:contract.bounds.max_imports) { Fail 'IMPORT_COUNT_EXCEEDED' $LogicalPath }

    foreach ($module in $imports) {
        $child = Resolve-InternalModule $module
        if ($null -ne $child) { Visit-Module $child ($Depth + 1) }
    }
    $script:visiting.Remove($LogicalPath)
    $script:visited[$LogicalPath] = $true
}

try {
    if (-not (Test-Path -LiteralPath $PackageRoot -PathType Container)) { Fail 'MISSING_PACKAGE_ROOT' }
    $script:packageRoot = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $PackageRoot).Path)
    Assert-NoReparsePath $script:packageRoot
    $content = Read-Xml 'content.xml' 'MISSING_CONTENT_MANIFEST'
    $ui = Read-Xml 'ui.xml' 'MISSING_UI_REGISTRATION'
    $contentDependencies = @($content.DocumentElement.SelectNodes('./dependency'))
    if ($content.DocumentElement.LocalName -cne 'content' -or
        $content.DocumentElement.GetAttribute('id') -cne $script:contract.package_id -or
        @($contentDependencies | Where-Object { $_.GetAttribute('id') -ceq $script:contract.required_content_dependency }).Count -ne 1) {
        Fail 'INVALID_PACKAGE_IDENTITY' 'content.xml'
    }
    if ($ui.DocumentElement.LocalName -cne 'addon' -or $ui.DocumentElement.GetAttribute('name') -cne $script:contract.package_id) {
        Fail 'MISSING_REGISTRATION' 'ui.xml'
    }
    $environment = @($ui.DocumentElement.SelectNodes('./environment') | Where-Object {
        $_.GetAttribute('type') -ceq $script:contract.required_environment
    })
    if ($environment.Count -ne 1) { Fail 'MISSING_REGISTRATION' 'ui.xml' }
    $files = @($environment[0].SelectNodes('./file'))
    if ($files.Count -ne 1) { Fail 'MISSING_REGISTRATION' 'ui.xml' }
    $uiDependencies = @($environment[0].SelectNodes('./dependency'))
    if (@($uiDependencies | Where-Object { $_.GetAttribute('name') -ceq $script:contract.required_ui_dependency }).Count -ne 1) {
        Fail 'MISSING_REGISTRATION' 'ui.xml'
    }
    $script:entrypoint = $files[0].GetAttribute('name')
    if ($script:entrypoint -cne $script:contract.required_entrypoint) { Fail 'WRONG_ENTRYPOINT' $script:entrypoint }

    Visit-Module $script:entrypoint 0
    $script:importGraph = @($script:visited.Keys | Sort-Object)
    $bindingPaths = @()
    $bindingCount = 0
    $bindingAccessCount = 0
    $alternateBinding = $false
    foreach ($logicalPath in $script:importGraph) {
        $source = $script:sources[$logicalPath]
        $tokens = @(Get-LuaTokens $source)
        $moduleBinding = Get-ExecutableFfiCAccesses $tokens
        $bindingAccessCount += $moduleBinding.AccessCount
        if ($moduleBinding.ApprovedCount -gt 0) {
            $bindingCount += $moduleBinding.ApprovedCount
            $bindingPaths += $logicalPath
        }
        elseif (Test-ExecutableAlternateBinding $tokens) { $alternateBinding = $true }
    }

    if ($bindingAccessCount -eq 0 -and $bindingCount -eq 0 -and -not $alternateBinding) {
        Fail 'NATIVE_BINDING_NOT_FOUND' $script:entrypoint
    }
    if ($bindingAccessCount -ne 1 -or $bindingCount -ne 1 -or $alternateBinding) {
        Fail 'ALTERNATE_BINDING_SOURCE' $script:entrypoint
    }
    Write-Output 'PASS package: live_galaxy (local static evidence)'
    foreach ($logicalPath in $script:importGraph) { Write-Output "IMPORT $logicalPath" }
    Write-Output "NATIVE $($bindingPaths[0])"
    exit 0
}
catch {
    Write-Output "FAIL package: $script:failureCode $($script:diagnostics -join ', ')"
    exit 1
}
