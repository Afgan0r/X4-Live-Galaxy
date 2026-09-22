Set-StrictMode -Version Latest

function Write-HeavyProfileLua(
    [string]$LimitsRaw,
    [string]$Target,
    [string]$FactionInventoryPath
) {
    $values = $LimitsRaw | ConvertFrom-Json
    if (-not ($values.PSObject.Properties.Name -ccontains 'heavy_profile_version')) { return $false }
    $rows = @($values.PSObject.Properties | Sort-Object Name | ForEach-Object {
        if ($_.Name -notmatch '^[a-z_]+$' -or $_.Value -isnot [long] -and $_.Value -isnot [int]) {
            throw 'HEAVY_LUA_PROFILE_SHAPE'
        }
        "    $($_.Name) = $($_.Value),"
    })
    $digest = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($LimitsRaw))).ToLowerInvariant()
    $prefix = @('local config = {}', "config.profile_sha256 = '$digest'")
    $suffix = @("    }, 'argon')", 'end', 'return config', '')
    if (-not [string]::IsNullOrWhiteSpace($FactionInventoryPath)) {
        $inventory = Get-Content -LiteralPath $FactionInventoryPath -Raw | ConvertFrom-Json
        if ($inventory.schema_version -ne 1 -or @($inventory.entries).Count -eq 0) {
            throw 'FACTION_INVENTORY_INVALID'
        }
        $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        $inventoryRows = [Collections.Generic.List[string]]::new()
        $included = [Collections.Generic.List[string]]::new()
        foreach ($entry in $inventory.entries) {
            if ($entry.id -notmatch '^[a-z0-9_-]{1,64}$' -or -not $seen.Add($entry.id) -or
                $entry.origin -notin @('vanilla', 'dlc', 'player', 'service', 'modded') -or
                $entry.evidence -notmatch '^[a-z0-9_:+-]{1,64}$') {
                throw 'FACTION_INVENTORY_INVALID'
            }
            $independent = if ($null -eq $entry.independent) { 'nil' } elseif ($entry.independent) { 'true' } else { 'false' }
            $mind = if ($entry.mind_candidate) { 'true' } else { 'false' }
            $inventoryRows.Add("    ['$($entry.id)'] = { origin = '$($entry.origin)', independent = $independent, mind_candidate = $mind, evidence = '$($entry.evidence)' },")
            if ($entry.independent -eq $true -and $entry.origin -in @('vanilla', 'dlc')) {
                $included.Add("'$($entry.id)'")
            }
        }
        $prefix += @('local inventory = {') + @($inventoryRows) + @('}', 'local factions = { ' + (@($included) -join ', ') + ' }')
        $suffix = @('    }, factions, inventory)', 'end', 'return config', '')
    }
    $lua = $prefix + @('function config.options()',
        "    return require('live_galaxy.lua.live_galaxy_ship_profile').options({") + $rows + $suffix
    [IO.File]::WriteAllText($Target, ($lua -join "`n"), [Text.UTF8Encoding]::new($false))
    return $true
}

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
