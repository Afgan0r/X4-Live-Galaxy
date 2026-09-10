Set-StrictMode -Version Latest

function Assert-CarrierBManifest($Manifest, [string]$ExpectedCandidate) {
    if ($Manifest.product -cne 'live_galaxy' -or $Manifest.product_version -cne '0.1.0' -or
        $Manifest.source_revision -notmatch '^[0-9a-f]{40}$' -or
        $Manifest.candidate -cne $ExpectedCandidate -or
        $Manifest.architecture -cne 'amd64-pe32+' -or
        $Manifest.initializer -cne 'luaopen_live_galaxy_carrier' -or
        $Manifest.native_abi_version -ne 2 -or $Manifest.control_contract_version -ne 3 -or
        $Manifest.envelope_contract_version -ne 2 -or $Manifest.semantic_versions.schema -ne 1 -or
        $Manifest.semantic_versions.policy -ne 2 -or
        $Manifest.semantic_versions.canonicalization -ne 3 -or
        $Manifest.semantic_versions.digest -ne 1) {
        throw 'MANIFEST_VERSION_OR_IDENTITY_INVALID'
    }
}

function Resolve-CarrierBManifestPath([string]$Root, [string]$Relative) {
    if ([string]::IsNullOrWhiteSpace($Relative) -or [IO.Path]::IsPathRooted($Relative) -or
        $Relative.Contains('\')) {
        throw "MANIFEST_PATH_INVALID:$Relative"
    }
    $rootPath = [IO.Path]::GetFullPath($Root)
    $fullPath = [IO.Path]::GetFullPath((Join-Path $rootPath $Relative))
    $prefix = $rootPath.TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    if (-not $fullPath.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "MANIFEST_PATH_OUTSIDE_PACKAGE:$Relative"
    }
    $canonical = [IO.Path]::GetRelativePath($rootPath, $fullPath).Replace('\', '/')
    if ($canonical -cne $Relative -or $canonical -ceq 'manifest.json') {
        throw "MANIFEST_PATH_NONCANONICAL:$Relative"
    }
    $fullPath
}

function Assert-CarrierBBundleFiles([string]$Root, $Manifest) {
    $manifestNames = @($Manifest.files.PSObject.Properties.Name)
    $actualNames = @(Get-ChildItem -LiteralPath $Root -Recurse -File | ForEach-Object {
        [IO.Path]::GetRelativePath($Root, $_.FullName).Replace('\', '/')
    } | Where-Object { $_ -cne 'manifest.json' } | Sort-Object)
    $expectedNames = @($manifestNames | Sort-Object)
    if ($actualNames.Count -ne $expectedNames.Count -or
        @($actualNames | Where-Object { $_ -cnotin $expectedNames }).Count -ne 0 -or
        @($expectedNames | Where-Object { $_ -cnotin $actualNames }).Count -ne 0) {
        throw 'MANIFEST_FILE_SET_MISMATCH'
    }
    foreach ($property in $Manifest.files.PSObject.Properties) {
        $path = Resolve-CarrierBManifestPath $Root $property.Name
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "MANIFEST_FILE_MISSING:$($property.Name)"
        }
        $actual = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -cne $property.Value) { throw "MANIFEST_HASH_MISMATCH:$($property.Name)" }
    }
}
