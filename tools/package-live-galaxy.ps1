[CmdletBinding()]
param(
    [switch]$SelfTest,
    [string]$OutputDirectory = 'dist/live-galaxy-carrier-b',
    [string]$LimitsFile
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$ownedNative = 'ui_c_library_live_galaxy_carrier_64.txt'
$requiredExport = 'luaopen_live_galaxy_carrier'
$limitFields = @(
    'complete_message_bytes', 'control_message_bytes', 'max_candidate_raw_bytes',
    'max_candidate_records', 'max_candidate_batches', 'max_candidate_work',
    'max_message_age_millis', 'max_message_inactivity_millis', 'max_candidates',
    'max_aggregate_bytes', 'max_aggregate_records', 'max_aggregate_batches',
    'max_aggregate_work', 'max_publication_records', 'max_publication_content_bytes',
    'max_pending_bytes', 'max_total_bytes', 'max_lifecycle_work',
    'max_delivery_attempts', 'max_blockers', 'reconnect_attempts',
    'reconnect_delay_millis', 'availability_interval_millis'
)

function Assert-Contained([string]$Path, [string]$Root) {
    $full = [IO.Path]::GetFullPath($Path)
    $base = [IO.Path]::GetFullPath($Root).TrimEnd('\') + '\'
    if (-not $full.StartsWith($base, [StringComparison]::OrdinalIgnoreCase)) {
        throw "OUTPUT_OUTSIDE_REPOSITORY:$full"
    }
    $full
}

function Read-U16([byte[]]$Bytes, [int]$Offset) {
    [BitConverter]::ToUInt16($Bytes, $Offset)
}

function Read-U32([byte[]]$Bytes, [int]$Offset) {
    [BitConverter]::ToUInt32($Bytes, $Offset)
}

function Resolve-Rva([byte[]]$Bytes, [int]$Pe, [uint32]$Rva) {
    $sections = Read-U16 $Bytes ($Pe + 6)
    $optionalSize = Read-U16 $Bytes ($Pe + 20)
    $section = $Pe + 24 + $optionalSize
    for ($index = 0; $index -lt $sections; $index++) {
        $entry = $section + ($index * 40)
        $virtualSize = Read-U32 $Bytes ($entry + 8)
        $virtualAddress = Read-U32 $Bytes ($entry + 12)
        $rawSize = Read-U32 $Bytes ($entry + 16)
        $rawOffset = Read-U32 $Bytes ($entry + 20)
        $span = [Math]::Max($virtualSize, $rawSize)
        if ($Rva -ge $virtualAddress -and $Rva -lt ($virtualAddress + $span)) {
            return [int]($rawOffset + ($Rva - $virtualAddress))
        }
    }
    throw 'PE_RVA_OUT_OF_RANGE'
}

function Read-CString([byte[]]$Bytes, [int]$Offset) {
    $end = $Offset
    while ($end -lt $Bytes.Length -and $Bytes[$end] -ne 0) { $end++ }
    if ($end -ge $Bytes.Length) { throw 'PE_STRING_UNTERMINATED' }
    [Text.Encoding]::ASCII.GetString($Bytes, $Offset, $end - $Offset)
}

function Assert-OwnedPe([string]$Path) {
    $bytes = [IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -lt 512 -or $bytes[0] -ne 0x4d -or $bytes[1] -ne 0x5a) {
        throw 'PE_SIGNATURE_INVALID'
    }
    $pe = [int](Read-U32 $bytes 0x3c)
    if ($pe + 136 -ge $bytes.Length -or (Read-U32 $bytes $pe) -ne 0x00004550) {
        throw 'PE_SIGNATURE_INVALID'
    }
    if ((Read-U16 $bytes ($pe + 4)) -ne 0x8664 -or (Read-U16 $bytes ($pe + 24)) -ne 0x020b) {
        throw 'PE_NOT_AMD64_PE32_PLUS'
    }
    $exportRva = Read-U32 $bytes ($pe + 24 + 112)
    if ($exportRva -eq 0) { throw 'PE_EXPORT_MISSING' }
    $export = Resolve-Rva $bytes $pe $exportRva
    $nameCount = Read-U32 $bytes ($export + 24)
    $namesRva = Read-U32 $bytes ($export + 32)
    if ($nameCount -ne 1) { throw "PE_EXPORT_COUNT:$nameCount" }
    $names = Resolve-Rva $bytes $pe $namesRva
    $nameRva = Read-U32 $bytes $names
    $name = Read-CString $bytes (Resolve-Rva $bytes $pe $nameRva)
    if ($name -cne $requiredExport) { throw "PE_EXPORT_UNEXPECTED:$name" }
}

function Read-Limits([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw 'LIMITS_FILE_MISSING' }
    $raw = Get-Content -LiteralPath $Path -Raw
    if ([Text.Encoding]::UTF8.GetByteCount($raw) -gt 4096) { throw 'LIMITS_FILE_TOO_LARGE' }
    $value = $raw | ConvertFrom-Json
    $names = @($value.PSObject.Properties.Name)
    if ($names.Count -ne $limitFields.Count -or @($names | Where-Object { $_ -cnotin $limitFields }).Count) {
        throw 'LIMITS_SHAPE_INVALID'
    }
    foreach ($field in $limitFields) {
        if ($value.$field -isnot [long] -and $value.$field -isnot [int]) { throw "LIMIT_NOT_INTEGER:$field" }
        if ($value.$field -le 0) { throw "LIMIT_NOT_POSITIVE:$field" }
    }
    $raw
}

function Assert-SourceRegistration([string]$ExtensionRoot) {
    $content = [xml](Get-Content -LiteralPath (Join-Path $ExtensionRoot 'content.xml') -Raw)
    $ui = [xml](Get-Content -LiteralPath (Join-Path $ExtensionRoot 'ui.xml') -Raw)
    if ($content.DocumentElement.GetAttribute('id') -cne 'live_galaxy' -or
        @($content.DocumentElement.SelectNodes('./dependency')).Count -ne 0) { throw 'CONTENT_REGISTRATION_INVALID' }
    $menus = @($ui.DocumentElement.SelectNodes('./environment') | Where-Object { $_.GetAttribute('type') -ceq 'menus' })
    if ($ui.DocumentElement.GetAttribute('name') -cne 'live_galaxy' -or $menus.Count -ne 1 -or
        @($menus[0].SelectNodes('./dependency')).Count -ne 0 -or
        @($menus[0].SelectNodes('./file')).Count -ne 1 -or
        $menus[0].SelectSingleNode('./file').GetAttribute('name') -cne 'lua/live_galaxy_runtime.lua') {
        throw 'UI_REGISTRATION_INVALID'
    }
    if (-not (Test-Path -LiteralPath (Join-Path $ExtensionRoot 'lua/live_galaxy_runtime.lua') -PathType Leaf)) {
        throw 'MISSING_ENTRYPOINT'
    }
    $nativeNames = @(Get-ChildItem -LiteralPath $ExtensionRoot -File |
        Where-Object { $_.Name -like 'ui_c_library_live_galaxy_carrier_64.txt' } |
        Select-Object -ExpandProperty Name)
    if ($nativeNames.Count -ne 1 -or $nativeNames[0] -cne $ownedNative) { throw 'NATIVE_PATH_OR_CASE_INVALID' }
}

function Assert-Manifest($Manifest) {
    if ($Manifest.product -cne 'live_galaxy' -or $Manifest.product_version -cne '0.1.0' -or
        $Manifest.architecture -cne 'amd64-pe32+' -or $Manifest.initializer -cne $requiredExport -or
        $Manifest.native_abi_version -ne 2 -or $Manifest.control_contract_version -ne 3 -or
        $Manifest.envelope_contract_version -ne 2 -or $Manifest.semantic_versions.schema -ne 1 -or
        $Manifest.semantic_versions.policy -ne 2 -or $Manifest.semantic_versions.canonicalization -ne 3 -or
        $Manifest.semantic_versions.digest -ne 1) { throw 'MANIFEST_VERSION_OR_IDENTITY_INVALID' }
}

function Assert-Bundle([string]$Root) {
    Assert-SourceRegistration (Join-Path $Root 'extensions/live_galaxy')
    Assert-OwnedPe (Join-Path $Root "extensions/live_galaxy/$ownedNative")
    if (-not (Test-Path -LiteralPath (Join-Path $Root 'live-galaxy-bridge.exe') -PathType Leaf)) {
        throw 'BRIDGE_MISSING'
    }
    $manifest = Get-Content -LiteralPath (Join-Path $Root 'manifest.json') -Raw | ConvertFrom-Json
    Assert-Manifest $manifest
    foreach ($property in $manifest.files.PSObject.Properties) {
        $path = Join-Path $Root $property.Name
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "MANIFEST_FILE_MISSING:$($property.Name)" }
        $actual = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -cne $property.Value) { throw "MANIFEST_HASH_MISMATCH:$($property.Name)" }
    }
}

function Write-Bundle([string]$Destination, [string]$LimitsPath, [bool]$Calibration) {
    $destination = Assert-Contained $Destination $repo
    $parent = Split-Path -Parent $destination
    [IO.Directory]::CreateDirectory($parent) | Out-Null
    $stage = Join-Path $parent ('.live-galaxy-stage-' + [guid]::NewGuid().ToString('N'))
    try {
        $extensionSource = Join-Path $repo 'extensions/live_galaxy'
        $nativeSource = Join-Path $repo 'target/release/x4_carrier_native.dll'
        $bridgeSource = Join-Path $repo 'target/release/x4-bridge.exe'
        foreach ($path in @($nativeSource, $bridgeSource)) {
            if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "BUILD_ARTIFACT_MISSING:$path" }
        }
        Assert-OwnedPe $nativeSource
        $limitsRaw = Read-Limits $LimitsPath
        $extensionTarget = Join-Path $stage 'extensions/live_galaxy'
        [IO.Directory]::CreateDirectory($extensionTarget) | Out-Null
        & (Join-Path $extensionSource 'tests/x4-package-conformance.ps1') -ExtensionRoot $extensionSource | Out-Null
        Get-ChildItem -LiteralPath $extensionSource | Where-Object { $_.Name -cne 'tests' } | ForEach-Object {
            Copy-Item -LiteralPath $_.FullName -Destination $extensionTarget -Recurse
        }
        Copy-Item -LiteralPath $nativeSource -Destination (Join-Path $extensionTarget $ownedNative)
        Copy-Item -LiteralPath $bridgeSource -Destination (Join-Path $stage 'live-galaxy-bridge.exe')
        [IO.File]::WriteAllText((Join-Path $stage 'carrier-b-limits.json'), $limitsRaw, [Text.UTF8Encoding]::new($false))
        $startup = @(
            'Live Galaxy Carrier B starts as two independent components.',
            'Start live-galaxy-bridge.exe --data-dir <private-directory> --limits-file carrier-b-limits.json.',
            'Install extensions/live_galaxy under the X4 game-root extensions directory.',
            'Then enable the unpacked extension with protected UI extensions permitted by X4.',
            'Starting X4 first is also supported; the bridge reconnects within the configured finite budget.',
            'Do not hot-replace the native image. Restart X4 after DLL, Lua, ABI, or contract changes.'
        ) -join [Environment]::NewLine
        [IO.File]::WriteAllText((Join-Path $stage 'STARTUP.txt'), $startup, [Text.UTF8Encoding]::new($false))
        Assert-SourceRegistration $extensionTarget
        Assert-OwnedPe (Join-Path $extensionTarget $ownedNative)
        $files = Get-ChildItem -LiteralPath $stage -Recurse -File | Sort-Object FullName
        $hashes = [ordered]@{}
        foreach ($file in $files) {
            $relative = [IO.Path]::GetRelativePath($stage, $file.FullName).Replace('\', '/')
            $hashes[$relative] = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        }
        $manifest = [ordered]@{
            product = 'live_galaxy'; product_version = '0.1.0'; source_revision = (git -C $repo rev-parse HEAD)
            candidate = $(if ($Calibration) { 'local-calibration-only' } else { 'ready-for-user-x4-checkpoint' })
            architecture = 'amd64-pe32+'; initializer = $requiredExport
            native_abi_version = 2; control_contract_version = 3; envelope_contract_version = 2
            semantic_versions = [ordered]@{ schema = 1; policy = 2; canonicalization = 3; digest = 1 }
            files = $hashes
        }
        $manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $stage 'manifest.json') -Encoding utf8NoBOM
        Assert-Bundle $stage
        if (Test-Path -LiteralPath $destination) {
            $old = "$destination.old"
            if (Test-Path -LiteralPath $old) { Remove-Item -LiteralPath $old -Recurse -Force }
            Move-Item -LiteralPath $destination -Destination $old
            try { Move-Item -LiteralPath $stage -Destination $destination }
            catch { Move-Item -LiteralPath $old -Destination $destination; throw }
            Remove-Item -LiteralPath $old -Recurse -Force
        } else { Move-Item -LiteralPath $stage -Destination $destination }
        return $destination
    } finally {
        if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
    }
}

function Invoke-SelfTest {
    cargo build --locked --release -p x4-carrier-native -p x4-bridge
    if ($LASTEXITCODE -ne 0) { throw 'RELEASE_BUILD_FAILED' }
    $scratch = Join-Path $repo ('tools/.cache/package-selftest-' + [guid]::NewGuid().ToString('N'))
    [IO.Directory]::CreateDirectory($scratch) | Out-Null
    try {
        $fixture = Join-Path $scratch 'limits.json'
        $values = [ordered]@{}
        foreach ($field in $limitFields) { $values[$field] = 1 }
        $values.complete_message_bytes = 2048; $values.control_message_bytes = 512
        $values.max_candidate_raw_bytes = 96; $values.max_pending_bytes = 4096
        $values.max_total_bytes = 8192; $values.max_aggregate_bytes = 4096
        $values.max_publication_content_bytes = 4096
        $values | ConvertTo-Json -Compress | Set-Content -LiteralPath $fixture -Encoding utf8NoBOM
        $selectedLimits = if ($LimitsFile) { Assert-Contained (Join-Path $repo $LimitsFile) $repo } else { $fixture }
        $bundle = Write-Bundle (Join-Path $scratch 'bundle') $selectedLimits (-not [bool]$LimitsFile)
        $manifest = Get-Content -LiteralPath (Join-Path $bundle 'manifest.json') -Raw | ConvertFrom-Json
        $wrongVersions = @{
            native_abi_version = @(1, 3)
            control_contract_version = @(1, 2, 4)
            envelope_contract_version = @(1, 3)
        }
        foreach ($field in $wrongVersions.Keys) {
            $saved = $manifest.$field
            foreach ($wrong in $wrongVersions[$field]) {
                $manifest.$field = $wrong
                try { Assert-Manifest $manifest; throw 'NEGATIVE_VERSION_ACCEPTED' }
                catch { if ($_.Exception.Message -eq 'NEGATIVE_VERSION_ACCEPTED') { throw } }
            }
            $manifest.$field = $saved
        }
        Assert-Bundle $bundle
        $wrongPe = Join-Path $scratch 'wrong.txt'; [IO.File]::WriteAllText($wrongPe, 'not-pe')
        try { Assert-OwnedPe $wrongPe; throw 'NEGATIVE_PE_ACCEPTED' } catch { if ($_.Exception.Message -eq 'NEGATIVE_PE_ACCEPTED') { throw } }
        $wrongExport = Join-Path $scratch 'wrong-export.txt'
        $image = [IO.File]::ReadAllBytes((Join-Path $repo 'target/release/x4_carrier_native.dll'))
        $needle = [Text.Encoding]::ASCII.GetBytes($requiredExport)
        for ($offset = 0; $offset -le $image.Length - $needle.Length; $offset++) {
            $match = $true
            for ($index = 0; $index -lt $needle.Length; $index++) {
                if ($image[$offset + $index] -ne $needle[$index]) { $match = $false; break }
            }
            if ($match) { $image[$offset] = [byte][char]'x' }
        }
        [IO.File]::WriteAllBytes($wrongExport, $image)
        try { Assert-OwnedPe $wrongExport; throw 'NEGATIVE_EXPORT_ACCEPTED' } catch { if ($_.Exception.Message -eq 'NEGATIVE_EXPORT_ACCEPTED') { throw } }
        $nativePath = Join-Path $bundle "extensions/live_galaxy/$ownedNative"
        $nativeBackup = "$nativePath.backup"; Move-Item -LiteralPath $nativePath -Destination $nativeBackup
        try { Assert-Bundle $bundle; throw 'NEGATIVE_PATH_ACCEPTED' } catch { if ($_.Exception.Message -eq 'NEGATIVE_PATH_ACCEPTED') { throw } }
        Move-Item -LiteralPath $nativeBackup -Destination $nativePath
        [IO.File]::AppendAllText((Join-Path $bundle 'STARTUP.txt'), 'tamper')
        try { Assert-Bundle $bundle; throw 'NEGATIVE_HASH_ACCEPTED' } catch { if ($_.Exception.Message -eq 'NEGATIVE_HASH_ACCEPTED') { throw } }
        $missing = Join-Path $scratch 'missing-output'
        [IO.Directory]::CreateDirectory($missing) | Out-Null
        [IO.File]::WriteAllText((Join-Path $missing 'sentinel'), 'preserve')
        try { Write-Bundle $missing (Join-Path $scratch 'absent.json') $true; throw 'NEGATIVE_LIMITS_ACCEPTED' }
        catch { if ($_.Exception.Message -eq 'NEGATIVE_LIMITS_ACCEPTED') { throw } }
        if ((Get-Content -LiteralPath (Join-Path $missing 'sentinel') -Raw) -cne 'preserve') { throw 'PREVIOUS_OUTPUT_NOT_PRESERVED' }
        Write-Output 'Package self-test: release PE, sole export, XML/layout, manifest versions, limits and atomic preservation passed.'
    } finally { if (Test-Path -LiteralPath $scratch) { Remove-Item -LiteralPath $scratch -Recurse -Force } }
}

if ($SelfTest) { Invoke-SelfTest; exit 0 }
if (-not $LimitsFile) { throw 'LIMITS_FILE_REQUIRED_FOR_READY_PACKAGE' }
$limits = Assert-Contained (Join-Path $repo $LimitsFile) $repo
$output = Assert-Contained (Join-Path $repo $OutputDirectory) $repo
cargo build --locked --release -p x4-carrier-native -p x4-bridge
if ($LASTEXITCODE -ne 0) { throw 'RELEASE_BUILD_FAILED' }
$ready = Write-Bundle $output $limits $false
Write-Output "READY_PACKAGE:$ready"
