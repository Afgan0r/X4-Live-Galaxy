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

function Read-BoundedBytes([string]$Path, [int]$Maximum, [string]$FailureCode) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { Fail $FailureCode }
    $bytes = [System.IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Path).Path)
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
    foreach ($segment in @($normalized -split '/')) {
        if ([string]::IsNullOrWhiteSpace($segment) -or $segment -eq '.') { Fail 'ROOT_ESCAPE' 'non-conformant' $normalized }
        $match = @(Get-ChildItem -LiteralPath $current -Force -ErrorAction SilentlyContinue | Where-Object { $_.Name -ceq $segment })
        if ($match.Count -ne 1) { Fail $MissingCode 'non-conformant' $normalized }
        $current = $match[0].FullName
    }
    $full = [IO.Path]::GetFullPath($current)
    $prefix = $script:packageRoot.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    if (-not $full.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) { Fail 'ROOT_ESCAPE' 'non-conformant' $normalized }
    return [pscustomobject]@{ FullPath = $full; LogicalPath = $normalized }
}

function Get-Imports([string]$Source) {
    $imports = [System.Collections.Generic.List[string]]::new()
    $constants = @{}
    foreach ($match in [regex]::Matches($Source, '(?m)^\s*local\s+([A-Z][A-Z0-9_]*)\s*=\s*["'']([^"'']+)["'']')) {
        $constants[$match.Groups[1].Value] = $match.Groups[2].Value
    }
    foreach ($match in [regex]::Matches($Source, 'require\s*\(\s*["'']([^"'']+)["'']\s*\)')) {
        $imports.Add($match.Groups[1].Value)
    }
    foreach ($match in [regex]::Matches($Source, 'pcall\s*\(\s*require\s*,\s*["'']([^"'']+)["'']\s*\)')) {
        $imports.Add($match.Groups[1].Value)
    }
    foreach ($match in [regex]::Matches($Source, 'pcall\s*\(\s*require\s*,\s*([A-Z][A-Z0-9_]*)\s*\)')) {
        if ($constants.ContainsKey($match.Groups[1].Value)) { $imports.Add($constants[$match.Groups[1].Value]) }
    }
    $helpers = @{}
    foreach ($match in [regex]::Matches($Source, '(?s)local\s+function\s+([a-z_][a-z0-9_]*)\s*\(\s*([a-z_][a-z0-9_]*)\s*\).*?return\s+require\s*\(\s*([A-Z][A-Z0-9_]*)\s*\.\.\s*\2\s*\).*?end')) {
        if ($constants.ContainsKey($match.Groups[3].Value)) { $helpers[$match.Groups[1].Value] = $constants[$match.Groups[3].Value] }
    }
    foreach ($helper in $helpers.GetEnumerator()) {
        $pattern = [regex]::Escape($helper.Key) + '\s*\(\s*["'']([^"'']+)["'']\s*\)'
        foreach ($match in [regex]::Matches($Source, $pattern)) { $imports.Add($helper.Value + $match.Groups[1].Value) }
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
    $script:classification = 'production-faithful'
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
