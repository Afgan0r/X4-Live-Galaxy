param(
    [Alias('Case')]
    [ValidateSet('all', 'reuse-contract')]
    [string]$ContractCase = 'all',
    [string]$PreparedBuildRoot,
    [string]$PreparedBuildKey
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$builderPath = Join-Path $root 'tools/x4-verification/build-candidate-extension.ps1'
$dispatcherPath = Join-Path $root 'tools/x4-verification/run-candidate-package.ps1'
$matrixPath = Join-Path $root 'tests/x4-candidates/phase-05.1-candidates.v1.json'
$publicRoot = Join-Path $root 'extensions/live_galaxy'

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Get-Digest([string]$Path) {
    [Convert]::ToHexString(
        [Security.Cryptography.SHA256]::HashData([IO.File]::ReadAllBytes($Path))
    ).ToLowerInvariant()
}

function Assert-OwnerOnly([string]$Path) {
    if ($IsWindows) {
        $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        $acl = Get-Acl -LiteralPath $Path
        Assert-True ($acl.GetOwner([Security.Principal.SecurityIdentifier]).Value -eq $sid) `
            'PREPARED_BUILD_OWNER_MISMATCH'
        foreach ($rule in @($acl.Access | Where-Object AccessControlType -eq 'Allow')) {
            $ruleSid = $rule.IdentityReference.Translate(
                [Security.Principal.SecurityIdentifier]
            ).Value
            Assert-True ($ruleSid -eq $sid) 'PREPARED_BUILD_NOT_OWNER_ONLY'
        }
        return
    }
    $mode = [IO.File]::GetUnixFileMode($Path)
    $forbidden = [IO.UnixFileMode]::GroupRead -bor [IO.UnixFileMode]::GroupWrite -bor `
        [IO.UnixFileMode]::GroupExecute -bor [IO.UnixFileMode]::OtherRead -bor `
        [IO.UnixFileMode]::OtherWrite -bor [IO.UnixFileMode]::OtherExecute
    Assert-True (($mode -band $forbidden) -eq 0) 'PREPARED_BUILD_NOT_OWNER_ONLY'
}

function Get-PreparedKey([string]$BuildRoot) {
    $matrixDigest = Get-Digest $matrixPath
    $material = [Collections.Generic.List[string]]::new()
    $material.Add("matrix=$matrixDigest")
    foreach ($sourcePath in @(
        'tools/x4-verification/build-candidate-extension.ps1',
        'tools/x4-verification/contracts/candidate-build-manifest.v1.json',
        'tools/x4-verification/templates/candidate-content.xml',
        'tools/x4-verification/templates/candidate-entry.lua',
        'tools/x4-verification/templates/candidate-ui.xml',
        'tests/x4-candidates/lua/live_galaxy_candidate_runner.lua'
    )) {
        $material.Add("source/$sourcePath=$(Get-Digest (Join-Path $root $sourcePath))")
    }
    $componentBindings = [ordered]@{
        dossier_digest = 'tools/x4-verification/contracts/phase-05.1-dossier.v1.json'
        registry_digest = 'tools/x4-verification/contracts/known-failures.v1.json'
        coverage_digest = 'tools/x4-verification/contracts/coverage.v1.json'
        runtime_evidence_schema_digest = 'tools/x4-verification/contracts/runtime-evidence.v1.json'
        owner_root_anchor_digest = 'tools/x4-verification/contracts/owner-root-anchor.v1.json'
        dispatcher_digest = 'tools/x4-verification/run-candidate-package.ps1'
        adapter_digest = 'tools/x4-verification/candidate-adapters.psm1'
        attestation_module_digest = 'tools/x4-verification/producer-attestation.psm1'
        bounded_reader_digest = 'tools/x4-verification/bounded-file.psm1'
        worker_digest = 'tools/x4-verification/isolation/candidate-worker.ps1'
        launcher_digest = 'tools/x4-verification/isolation/invoke-candidate-worker.ps1'
        worker_protocol_digest = 'tools/x4-verification/contracts/candidate-worker-protocol.v1.json'
    }
    $groups = @(Get-ChildItem -LiteralPath $BuildRoot -Directory | Sort-Object Name)
    Assert-True ($groups.Count -gt 0 -and $groups.Count -le 16) 'PREPARED_BUILD_GROUPS_INVALID'
    foreach ($group in $groups) {
        Assert-True (($group.Attributes -band [IO.FileAttributes]::ReparsePoint) -eq 0) `
            'PREPARED_BUILD_REPARSE_REJECTED'
        $manifestPath = Join-Path $group.FullName 'manifest/build-manifest.v1.json'
        Assert-True (Test-Path -LiteralPath $manifestPath -PathType Leaf) `
            'PREPARED_BUILD_MANIFEST_MISSING'
        $manifestItem = Get-Item -LiteralPath $manifestPath -Force
        Assert-True ($manifestItem.Length -le 262144 -and
            ($manifestItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -eq 0) `
            'PREPARED_BUILD_MANIFEST_INVALID'
        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json -Depth 64
        Assert-True ($manifest.matrix_digest -eq $matrixDigest) 'PREPARED_BUILD_SOURCE_MISMATCH'
        foreach ($binding in $componentBindings.GetEnumerator()) {
            Assert-True ($manifest.($binding.Key) -eq (Get-Digest (Join-Path $root $binding.Value))) `
                'PREPARED_BUILD_SOURCE_MISMATCH'
        }
        $material.Add("manifest/$($group.Name)=$(Get-Digest $manifestPath)")
        $generatedFiles = @($manifest.generated_files | Sort-Object path)
        Assert-True ($generatedFiles.Count -gt 0 -and $generatedFiles.Count -le 16) `
            'PREPARED_BUILD_FILES_INVALID'
        [long]$totalBytes = 0
        foreach ($generated in $generatedFiles) {
            $logicalPath = [string]$generated.path
            Assert-True ($logicalPath -match '^[a-zA-Z0-9._/-]+$' -and
                @($logicalPath -split '[\\/]+') -notcontains '..') `
                'PREPARED_BUILD_PATH_INVALID'
            $generatedPath = [IO.Path]::GetFullPath((Join-Path $group.FullName $logicalPath))
            Assert-True ($generatedPath.StartsWith(
                $group.FullName.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar,
                [StringComparison]::OrdinalIgnoreCase
            )) 'PREPARED_BUILD_PATH_INVALID'
            Assert-True (Test-Path -LiteralPath $generatedPath -PathType Leaf) `
                'PREPARED_BUILD_FILE_MISSING'
            $item = Get-Item -LiteralPath $generatedPath
            Assert-True ($item.Length -le 65536 -and
                ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -eq 0) `
                'PREPARED_BUILD_FILE_INVALID'
            $totalBytes += $item.Length
            $digest = Get-Digest $generatedPath
            Assert-True ($digest -eq $generated.sha256 -and $item.Length -eq $generated.bytes) `
                'PREPARED_BUILD_DIGEST_MISMATCH'
            $material.Add("$($group.Name)/$($generated.path)=$digest")
        }
        Assert-True ($totalBytes -le 524288) 'PREPARED_BUILD_FILES_INVALID'
    }
    return [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData(
        [Text.Encoding]::UTF8.GetBytes(($material -join "`n"))
    )).ToLowerInvariant()
}

function Assert-PreparedBuild([string]$BuildRoot, [string]$BuildKey) {
    Assert-True ($BuildKey -match '^[a-f0-9]{64}$') 'PREPARED_BUILD_KEY_INVALID'
    Assert-True (Test-Path -LiteralPath $BuildRoot -PathType Container) `
        'PREPARED_BUILD_MISSING'
    $full = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $BuildRoot).Path)
    $temp = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    Assert-True ($full.StartsWith($temp, [StringComparison]::OrdinalIgnoreCase)) `
        'PREPARED_BUILD_OUTSIDE_TEMP'
    $relative = [IO.Path]::GetRelativePath($temp, $full)
    $current = $temp
    foreach ($segment in @($relative -split '[\\/]+')) {
        $current = Join-Path $current $segment
        $item = Get-Item -LiteralPath $current -Force
        Assert-True (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -eq 0) `
            'PREPARED_BUILD_REPARSE_REJECTED'
    }
    Assert-OwnerOnly $full
    Assert-True ((Split-Path -Leaf $full) -eq $BuildKey) 'PREPARED_BUILD_IDENTITY_MISMATCH'
    Assert-True ((Get-PreparedKey $full) -eq $BuildKey) 'PREPARED_BUILD_KEY_MISMATCH'
}

function Copy-PreparedBuild([string]$Destination) {
    Assert-PreparedBuild $PreparedBuildRoot $PreparedBuildKey
    Copy-Item -LiteralPath $PreparedBuildRoot -Destination $Destination -Recurse
    if ($IsWindows) {
        $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        $null = & icacls.exe $Destination /inheritance:r /grant:r "*$sid`:(OI)(CI)F"
        Assert-True ($LASTEXITCODE -eq 0) 'PREPARED_BUILD_PERMISSION_FAILED'
        foreach ($item in @(Get-ChildItem -LiteralPath $Destination -Recurse -Force)) {
            $grant = if ($item.PSIsContainer) { "*$sid`:(OI)(CI)F" } else { "*$sid`:F" }
            $null = & icacls.exe $item.FullName /inheritance:r /grant:r $grant
            Assert-True ($LASTEXITCODE -eq 0) 'PREPARED_BUILD_PERMISSION_FAILED'
        }
    }
    else {
        [IO.File]::SetUnixFileMode(
            $Destination,
            [IO.UnixFileMode]::UserRead -bor [IO.UnixFileMode]::UserWrite -bor
                [IO.UnixFileMode]::UserExecute
        )
        foreach ($item in @(Get-ChildItem -LiteralPath $Destination -Recurse -Force)) {
            [IO.File]::SetUnixFileMode(
                $item.FullName,
                $(if ($item.PSIsContainer) {
                    [IO.UnixFileMode]::UserRead -bor [IO.UnixFileMode]::UserWrite -bor
                        [IO.UnixFileMode]::UserExecute
                } else {
                    [IO.UnixFileMode]::UserRead -bor [IO.UnixFileMode]::UserWrite
                })
            )
        }
    }
    Assert-PreparedBuild $Destination $PreparedBuildKey
    Assert-True ((Get-PreparedKey $PreparedBuildRoot) -eq $PreparedBuildKey) `
        'PREPARED_BUILD_SOURCE_CHANGED'
}

function Remove-PreparedClone([string]$Path) {
    try { Remove-Item -LiteralPath $Path -Recurse -Force }
    catch { throw 'PREPARED_BUILD_CLONE_CLEANUP_FAILED' }
    if (Test-Path -LiteralPath $Path) { throw 'PREPARED_BUILD_CLONE_CLEANUP_FAILED' }
}

$hasPreparedBuild = -not [string]::IsNullOrWhiteSpace($PreparedBuildRoot) -or
    -not [string]::IsNullOrWhiteSpace($PreparedBuildKey)
Assert-True (-not $hasPreparedBuild -or (
    -not [string]::IsNullOrWhiteSpace($PreparedBuildRoot) -and
    -not [string]::IsNullOrWhiteSpace($PreparedBuildKey)
)) 'PREPARED_BUILD_PARAMETERS_INCOMPLETE'

function Update-CallerOwnedManifestDigests([string]$GroupRoot, $Manifest) {
    foreach ($row in @($Manifest.generated_files)) {
        $path = Join-Path $GroupRoot ([string]$row.path)
        $row.bytes = (Get-Item -LiteralPath $path).Length
        $row.sha256 = Get-Digest $path
    }
    $componentPaths = [ordered]@{
        adapter_digest = 'tools/x4-verification/candidate-adapters.psm1'
        attestation_module_digest = 'tools/x4-verification/producer-attestation.psm1'
        worker_digest = 'tools/x4-verification/isolation/candidate-worker.ps1'
        launcher_digest = 'tools/x4-verification/isolation/invoke-candidate-worker.ps1'
        worker_protocol_digest = 'tools/x4-verification/contracts/candidate-worker-protocol.v1.json'
        runtime_evidence_schema_digest = 'tools/x4-verification/contracts/runtime-evidence.v1.json'
        owner_root_anchor_digest = 'tools/x4-verification/contracts/owner-root-anchor.v1.json'
    }
    foreach ($binding in $componentPaths.GetEnumerator()) {
        $Manifest.($binding.Key) = Get-Digest (Join-Path $GroupRoot $binding.Value)
    }
    $graphMaterial =
        (Get-Digest (Join-Path $GroupRoot 'content.xml')) +
        (Get-Digest (Join-Path $GroupRoot 'ui.xml')) +
        (Get-Digest (Join-Path $GroupRoot 'lua/live_galaxy_candidate_entry.lua'))
    $Manifest.package_conformance.graph_digest = [Convert]::ToHexString(
        [Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($graphMaterial))
    ).ToLowerInvariant()
    $packageBytes = [Text.Encoding]::UTF8.GetBytes(
        ($Manifest.package_conformance | ConvertTo-Json -Compress -Depth 8)
    )
    $Manifest.package_conformance_digest = [Convert]::ToHexString(
        [Security.Cryptography.SHA256]::HashData($packageBytes)
    ).ToLowerInvariant()
}

function Invoke-Dispatcher([string]$GroupRoot, [string]$OutputPath, [int]$ExpectedExit) {
    $text = & pwsh -NoProfile -File $dispatcherPath -GroupRoot $GroupRoot -OutputPath $OutputPath 2>&1 | Out-String
    Assert-True ($LASTEXITCODE -eq $ExpectedExit) "Dispatcher exit $LASTEXITCODE, expected $ExpectedExit`: $text"
    return $text.Trim() | ConvertFrom-Json -DateKind String
}

if ($ContractCase -eq 'reuse-contract') {
    Assert-True $hasPreparedBuild 'PREPARED_BUILD_REQUIRED'
    $probeRoot = Join-Path ([IO.Path]::GetTempPath()) `
        ('live-galaxy-adversarial-reuse-' + [guid]::NewGuid().ToString('N'))
    $null = New-Item -ItemType Directory -Path $probeRoot
    if ($IsWindows) {
        $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        $null = & icacls.exe $probeRoot /inheritance:r /grant:r "*$sid`:(OI)(CI)F"
        Assert-True ($LASTEXITCODE -eq 0) 'PREPARED_BUILD_PERMISSION_FAILED'
    }
    $clone = Join-Path $probeRoot $PreparedBuildKey
    try {
        Copy-PreparedBuild $clone
        $manifestPath = Get-ChildItem -LiteralPath $clone -Recurse -Filter 'build-manifest.v1.json' |
            Select-Object -First 1 -ExpandProperty FullName
        $cleanupRejected = $false
        if ($IsWindows) {
            $lock = [IO.File]::Open($manifestPath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::None)
            try { Remove-PreparedClone $clone }
            catch { $cleanupRejected = $_.Exception.Message -eq 'PREPARED_BUILD_CLONE_CLEANUP_FAILED' }
            finally { $lock.Dispose() }
        }
        else {
            $originalMode = [IO.File]::GetUnixFileMode($probeRoot)
            [IO.File]::SetUnixFileMode(
                $probeRoot,
                [IO.UnixFileMode]::UserRead -bor [IO.UnixFileMode]::UserExecute
            )
            try { Remove-PreparedClone $clone }
            catch { $cleanupRejected = $_.Exception.Message -eq 'PREPARED_BUILD_CLONE_CLEANUP_FAILED' }
            finally { [IO.File]::SetUnixFileMode($probeRoot, $originalMode) }
        }
        Assert-True $cleanupRejected 'PREPARED_BUILD_CLEANUP_FAILURE_NOT_REPORTED'
        Remove-PreparedClone $clone
        Assert-PreparedBuild $PreparedBuildRoot $PreparedBuildKey
        Write-Output 'PASS: prepared-build adversarial reuse contract'
    }
    finally {
        if (Test-Path -LiteralPath $clone) { Remove-Item -LiteralPath $clone -Recurse -Force }
        if (Test-Path -LiteralPath $probeRoot) { Remove-Item -LiteralPath $probeRoot -Recurse -Force }
    }
    exit 0
}

$parameters = (Get-Command $dispatcherPath).Parameters.Keys
foreach ($forbidden in @('RootPath', 'TrustRootPath', 'CertificatePath', 'KeyName', 'TestMode', 'Command', 'ModulePath', 'WorkerPath')) {
    Assert-True ($parameters -notcontains $forbidden) "Production dispatcher exposes forbidden selector '$forbidden'."
}

$scratch = Join-Path ([IO.Path]::GetTempPath()) ('live-galaxy-plan08-adversarial-' + [guid]::NewGuid().ToString('N'))
$buildRoot = Join-Path $scratch $(if ($hasPreparedBuild) { $PreparedBuildKey } else { 'builds' })
$outputRoot = Join-Path $scratch 'output'
$null = New-Item -ItemType Directory -Path $outputRoot -Force
if ($IsWindows) {
    $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    $null = & icacls.exe $scratch /inheritance:r /grant:r "*$sid`:(OI)(CI)F"
    Assert-True ($LASTEXITCODE -eq 0) 'Unable to protect package-adversarial scratch fixture.'
    $null = & icacls.exe $outputRoot /inheritance:r /grant:r "*$sid`:(OI)(CI)F"
    Assert-True ($LASTEXITCODE -eq 0) 'Unable to protect package-adversarial output fixture.'
}
try {
    if ($hasPreparedBuild) {
        Copy-PreparedBuild $buildRoot
    }
    else {
        $builderText = & pwsh -NoProfile -File $builderPath -BuildRoot $buildRoot -MatrixPath $matrixPath 2>&1 | Out-String
        Assert-True ($LASTEXITCODE -eq 0) "Builder failed: $builderText"
    }
    $groupRoot = Join-Path $buildRoot 'p051-build-lifecycle'

    $baselinePath = Join-Path $outputRoot 'baseline.jsonl'
    $baseline = Invoke-Dispatcher $groupRoot $baselinePath 0
    Assert-True ($baseline.local_process_ready -eq $true -and $baseline.retainable -eq $false) 'Baseline local readiness or authority status is false.'
    $baselineBytes = [IO.File]::ReadAllBytes($baselinePath)
    $baselineDigest = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($baselineBytes)).ToLowerInvariant()
    $tamperedBytes = [byte[]]$baselineBytes.Clone()
    $tamperedBytes[0] = $tamperedBytes[0] -bxor 1
    Assert-True (([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($tamperedBytes)).ToLowerInvariant()) -ne $baselineDigest) 'Same-length JSONL tamper did not change the evidence digest.'

    $swapAdapterPath = Join-Path $groupRoot 'tools/x4-verification/candidate-adapters.psm1'
    $swapAdapterBytes = [IO.File]::ReadAllBytes($swapAdapterPath)
    $executedMarker = Join-Path $scratch 'unverified-component-executed.marker'
    $swapDone = Join-Path $scratch 'component-swap-completed.marker'
    $watcher = [IO.FileSystemWatcher]::new([IO.Path]::GetTempPath())
    $watcher.IncludeSubdirectories = $true
    $watcher.NotifyFilter = [IO.NotifyFilters]::DirectoryName
    $watcher.EnableRaisingEvents = $true
    $subscription = Register-ObjectEvent -InputObject $watcher -EventName Created `
        -MessageData ([pscustomobject]@{
            AdapterPath = $swapAdapterPath; Original = $swapAdapterBytes
            ExecutedMarker = $executedMarker; SwapDone = $swapDone
        }) -Action {
            if ($Event.SourceEventArgs.FullPath.EndsWith('verified-snapshot', [StringComparison]::OrdinalIgnoreCase)) {
                $data = $Event.MessageData
                $originalText = [Text.Encoding]::UTF8.GetString([byte[]]$data.Original)
                $probe = "[IO.File]::WriteAllText('$($data.ExecutedMarker.Replace("'", "''"))','executed')`n" + $originalText
                [IO.File]::WriteAllText($data.AdapterPath, $probe, [Text.UTF8Encoding]::new($false))
                [IO.File]::WriteAllText($data.SwapDone, 'done', [Text.UTF8Encoding]::new($false))
            }
        }
    try {
        $swapOutputPath = Join-Path $outputRoot 'swap-race.jsonl'
        $swapText = @(& pwsh -NoProfile -File $dispatcherPath -GroupRoot $groupRoot `
            -OutputPath $swapOutputPath 2>&1)
        $swapExit = $LASTEXITCODE
        for ($attempt = 0; $attempt -lt 30 -and -not (Test-Path -LiteralPath $swapDone); $attempt += 1) {
            Start-Sleep -Milliseconds 100
        }
        Assert-True (Test-Path -LiteralPath $swapDone) 'Synchronized component swap did not execute.'
        Assert-True (-not (Test-Path -LiteralPath $executedMarker)) 'Unverified swapped component bytes executed.'
        if ($swapExit -eq 0) {
            Assert-True (Test-Path -LiteralPath $swapOutputPath -PathType Leaf) 'Safe snapshot run omitted evidence.'
            foreach ($rowText in @(Get-Content -LiteralPath $swapOutputPath)) {
                $row = $rowText | ConvertFrom-Json
                Assert-True ($row.execution_verdict -eq 'pass' -and $row.effect_verdict -eq 'pass') `
                    'Safe snapshot run changed candidate semantics.'
            }
        }
        else {
            $swapResult = @($swapText | Where-Object { $_.ToString().TrimStart().StartsWith('{') })[-1] |
                ConvertFrom-Json -Depth 8
            Assert-True ($swapResult.reason_code -in @('PATH_IDENTITY_CHANGED', 'COMPONENT_DIGEST_MISMATCH', 'SNAPSHOT_DIGEST_MISMATCH')) `
                "Synchronized swap returned unstable rejection: $($swapResult.reason_code)"
            Assert-True (-not (Test-Path -LiteralPath $swapOutputPath)) 'Rejected swap left an evidence artifact.'
        }
    }
    finally {
        [IO.File]::WriteAllBytes($swapAdapterPath, $swapAdapterBytes)
        Unregister-Event -SubscriptionId $subscription.Id -ErrorAction SilentlyContinue
        Remove-Job -Id $subscription.Id -Force -ErrorAction SilentlyContinue
        $watcher.Dispose()
    }

    $manifestPath = Join-Path $groupRoot 'manifest/build-manifest.v1.json'
    $manifestText = Get-Content -LiteralPath $manifestPath -Raw
    try {
        $manifest = $manifestText | ConvertFrom-Json -Depth 64 -DateKind String
        $manifest.package_conformance.graph_digest = '0' * 64
        [IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 64), [Text.UTF8Encoding]::new($false))
        $forged = Invoke-Dispatcher $groupRoot (Join-Path $outputRoot 'forged-conformance.jsonl') 1
        Assert-True ($forged.local_process_ready -eq $false) 'Forged package conformance produced readiness.'
    }
    finally { [IO.File]::WriteAllText($manifestPath, $manifestText, [Text.UTF8Encoding]::new($false)) }

    $adapterPath = Join-Path $groupRoot 'tools/x4-verification/candidate-adapters.psm1'
    $adapterText = Get-Content -LiteralPath $adapterPath -Raw
    try {
        [IO.File]::AppendAllText($adapterPath, "`n# tampered`n", [Text.UTF8Encoding]::new($false))
        $tampered = Invoke-Dispatcher $groupRoot (Join-Path $outputRoot 'tampered-component.jsonl') 1
        Assert-True ($tampered.reason_code -eq 'COMPONENT_DIGEST_MISMATCH') 'Changed adapter digest was not rejected.'
    }
    finally { [IO.File]::WriteAllText($adapterPath, $adapterText, [Text.UTF8Encoding]::new($false)) }

    $manifestPath = Join-Path $groupRoot 'manifest/build-manifest.v1.json'
    $manifestText = Get-Content -LiteralPath $manifestPath -Raw
    $coordinatedMarker = Join-Path $scratch 'coordinated-forgery-executed.marker'
    $coordinatedOutput = Join-Path $outputRoot 'coordinated-forgery.jsonl'
    try {
        $forgedAdapter =
            "[IO.File]::WriteAllText('$($coordinatedMarker.Replace("'", "''"))','executed')`n" +
            $adapterText
        [IO.File]::WriteAllText($adapterPath, $forgedAdapter, [Text.UTF8Encoding]::new($false))
        [byte[]]$forgedBytes = [IO.File]::ReadAllBytes($adapterPath)
        $forgedDigest = [Convert]::ToHexString(
            [Security.Cryptography.SHA256]::HashData($forgedBytes)
        ).ToLowerInvariant()
        $manifest = $manifestText | ConvertFrom-Json -Depth 64 -DateKind String
        $manifest.adapter_digest = $forgedDigest
        $adapterRow = @($manifest.generated_files | Where-Object {
            $_.path -ceq 'tools/x4-verification/candidate-adapters.psm1'
        })[0]
        $adapterRow.bytes = $forgedBytes.Length
        $adapterRow.sha256 = $forgedDigest
        [IO.File]::WriteAllText(
            $manifestPath,
            ($manifest | ConvertTo-Json -Depth 64),
            [Text.UTF8Encoding]::new($false)
        )
        if ($IsWindows) {
            $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
            $null = & icacls.exe $adapterPath /inheritance:r /grant:r "*$sid`:F"
            Assert-True ($LASTEXITCODE -eq 0) 'Unable to protect forged adapter fixture.'
            $null = & icacls.exe $manifestPath /inheritance:r /grant:r "*$sid`:F"
            Assert-True ($LASTEXITCODE -eq 0) 'Unable to protect forged manifest fixture.'
        }
        $coordinated = Invoke-Dispatcher $groupRoot $coordinatedOutput 1
        Assert-True ($coordinated.reason_code -eq 'COMPONENT_DIGEST_MISMATCH') `
            "Coordinated component and manifest forgery returned '$($coordinated.reason_code)'."
        Assert-True (-not (Test-Path -LiteralPath $coordinatedMarker)) `
            'Coordinated forged component executed before rejection.'
        Assert-True (-not (Test-Path -LiteralPath $coordinatedOutput)) `
            'Coordinated forgery published an evidence artifact.'
    }
    finally {
        [IO.File]::WriteAllText($adapterPath, $adapterText, [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($manifestPath, $manifestText, [Text.UTF8Encoding]::new($false))
    }

    $loadedByteCases = @(
        @{ Name = 'content'; Path = 'content.xml'; Mutation = "`n<!-- coordinated forgery -->`n" },
        @{ Name = 'ui'; Path = 'ui.xml'; Mutation = "`n<!-- coordinated forgery -->`n" },
        @{ Name = 'entrypoint'; Path = 'lua/live_galaxy_candidate_entry.lua'; Mutation = "`nlocal forged_entry = true`n" },
        @{ Name = 'runner'; Path = 'lua/live_galaxy_candidate_runner.lua'; Mutation = "`nlocal forged_runner = true`n" }
    )
    foreach ($case in $loadedByteCases) {
        $loadedPath = Join-Path $groupRoot $case.Path
        $loadedText = Get-Content -LiteralPath $loadedPath -Raw
        $forgedOutput = Join-Path $outputRoot "coordinated-$($case.Name)-forgery.jsonl"
        try {
            [IO.File]::WriteAllText(
                $loadedPath,
                $loadedText + $case.Mutation,
                [Text.UTF8Encoding]::new($false)
            )
            $manifest = $manifestText | ConvertFrom-Json -Depth 64 -DateKind String
            $loadedRow = @($manifest.generated_files | Where-Object { $_.path -ceq $case.Path })[0]
            $loadedRow.bytes = (Get-Item -LiteralPath $loadedPath).Length
            $loadedRow.sha256 = Get-Digest $loadedPath
            $graphMaterial =
                (Get-Digest (Join-Path $groupRoot 'content.xml')) +
                (Get-Digest (Join-Path $groupRoot 'ui.xml')) +
                (Get-Digest (Join-Path $groupRoot 'lua/live_galaxy_candidate_entry.lua'))
            $manifest.package_conformance.graph_digest = [Convert]::ToHexString(
                [Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($graphMaterial))
            ).ToLowerInvariant()
            $packageBytes = [Text.Encoding]::UTF8.GetBytes(
                ($manifest.package_conformance | ConvertTo-Json -Compress -Depth 8)
            )
            $manifest.package_conformance_digest = [Convert]::ToHexString(
                [Security.Cryptography.SHA256]::HashData($packageBytes)
            ).ToLowerInvariant()
            [IO.File]::WriteAllText(
                $manifestPath,
                ($manifest | ConvertTo-Json -Depth 64),
                [Text.UTF8Encoding]::new($false)
            )
            $forged = Invoke-Dispatcher $groupRoot $forgedOutput 1
            Assert-True ($forged.reason_code -eq 'COMPONENT_DIGEST_MISMATCH') `
                "Coordinated $($case.Name) forgery returned '$($forged.reason_code)'."
            Assert-True (-not (Test-Path -LiteralPath $forgedOutput)) `
                "Coordinated $($case.Name) forgery published evidence."
            Assert-True (-not (Test-Path -LiteralPath "$forgedOutput.attestation.json")) `
                "Coordinated $($case.Name) forgery published attestation."
        }
        finally {
            [IO.File]::WriteAllText($loadedPath, $loadedText, [Text.UTF8Encoding]::new($false))
            [IO.File]::WriteAllText($manifestPath, $manifestText, [Text.UTF8Encoding]::new($false))
        }
    }

    $generatedJsonCases = @(
        @{
            Name = 'subset-whitespace'; Path = 'manifest/candidate-matrix-subset.v1.json'
            Mutate = { param([string]$Text) $Text + "`n " }
        },
        @{
            Name = 'contract-property-order-and-escape'; Path = 'manifest/package-conformance.v1.json'
            Mutate = {
                param([string]$Text)
                $value = $Text | ConvertFrom-Json -Depth 64 -DateKind String
                $reordered = [ordered]@{}
                foreach ($property in @($value.PSObject.Properties) | Sort-Object Name -Descending) {
                    $reordered[$property.Name] = $property.Value
                }
                ($reordered | ConvertTo-Json -Compress -Depth 64).
                    Replace('"contract_id":"candidate-', '"contract_id":"\u0063andidate-')
            }
        },
        @{
            Name = 'contract-duplicate-key'; Path = 'manifest/package-conformance.v1.json'
            Mutate = {
                param([string]$Text)
                $value = $Text | ConvertFrom-Json -Depth 64 -DateKind String
                $Text.Insert(1, '"schema_version":"' + $value.schema_version + '",')
            }
        }
    )
    foreach ($case in $generatedJsonCases) {
        $generatedPath = Join-Path $groupRoot $case.Path
        $generatedText = Get-Content -LiteralPath $generatedPath -Raw
        $forgedOutput = Join-Path $outputRoot "$($case.Name)-forgery.jsonl"
        try {
            [IO.File]::WriteAllText(
                $generatedPath,
                (& $case.Mutate $generatedText),
                [Text.UTF8Encoding]::new($false)
            )
            $manifest = $manifestText | ConvertFrom-Json -Depth 64 -DateKind String
            Update-CallerOwnedManifestDigests $groupRoot $manifest
            [IO.File]::WriteAllText(
                $manifestPath,
                ($manifest | ConvertTo-Json -Depth 64),
                [Text.UTF8Encoding]::new($false)
            )
            $forged = Invoke-Dispatcher $groupRoot $forgedOutput 1
            Assert-True ($forged.reason_code -eq 'COMPONENT_DIGEST_MISMATCH') `
                "Coordinated $($case.Name) and manifest forgery was not rejected."
            Assert-True (-not (Test-Path -LiteralPath $forgedOutput)) `
                "Coordinated $($case.Name) forgery published evidence."
            Assert-True (-not (Test-Path -LiteralPath "$forgedOutput.attestation.json")) `
                "Coordinated $($case.Name) forgery published attestation."
        }
        finally {
            [IO.File]::WriteAllText($generatedPath, $generatedText, [Text.UTF8Encoding]::new($false))
            [IO.File]::WriteAllText($manifestPath, $manifestText, [Text.UTF8Encoding]::new($false))
        }
    }

    $anchorPath = Join-Path $groupRoot 'tools/x4-verification/contracts/owner-root-anchor.v1.json'
    $anchorText = Get-Content -LiteralPath $anchorPath -Raw
    try {
        $anchor = $anchorText | ConvertFrom-Json -Depth 16 -DateKind String
        $anchor.status = 'configured'
        $anchor.root_spki_sha256 = '0' * 64
        [IO.File]::WriteAllText($anchorPath, ($anchor | ConvertTo-Json -Depth 16), [Text.UTF8Encoding]::new($false))
        $swappedRoot = Invoke-Dispatcher $groupRoot (Join-Path $outputRoot 'swapped-root.jsonl') 1
        Assert-True ($swappedRoot.reason_code -eq 'COMPONENT_DIGEST_MISMATCH') 'Fresh, swapped, or test root escaped the repository digest pin.'
    }
    finally { [IO.File]::WriteAllText($anchorPath, $anchorText, [Text.UTF8Encoding]::new($false)) }

    $subsetPath = Join-Path $groupRoot 'manifest/candidate-matrix-subset.v1.json'
    $subsetText = Get-Content -LiteralPath $subsetPath -Raw
    try {
        $subset = $subsetText | ConvertFrom-Json -Depth 64 -DateKind String
        $subset.candidates[0].id = 'local-contract-success'
        [IO.File]::WriteAllText($subsetPath, ($subset | ConvertTo-Json -Depth 64), [Text.UTF8Encoding]::new($false))
        $escaped = Invoke-Dispatcher $groupRoot (Join-Path $outputRoot 'adapter-escape.jsonl') 1
        Assert-True ($escaped.local_process_ready -eq $false) 'Arbitrary adapter identity produced readiness.'
    }
    finally { [IO.File]::WriteAllText($subsetPath, $subsetText, [Text.UTF8Encoding]::new($false)) }

    $publicOutput = Join-Path $publicRoot 'candidate-evidence.jsonl'
    $public = Invoke-Dispatcher $groupRoot $publicOutput 1
    Assert-True ($public.reason_code -eq 'OUTPUT_DESTINATION_REJECTED') 'Public package output destination was accepted.'
    Assert-True (-not (Test-Path -LiteralPath $publicOutput)) 'Dispatcher wrote into the public package.'

    foreach ($destinationCase in @(
        @{ Name = 'installed game'; Path = Join-Path $scratch 'steamapps/common/X4 Foundations/extensions/live_galaxy/candidate.jsonl' },
        @{ Name = 'public runtime'; Path = Join-Path $scratch 'staging/extensions/live_galaxy/candidate.jsonl' },
        @{ Name = 'X4 save'; Path = Join-Path $scratch 'Documents/Egosoft/X4/123456/save/candidate.jsonl' }
    )) {
        $parent = Split-Path -Parent $destinationCase.Path
        $null = New-Item -ItemType Directory -Path $parent -Force
        $rejectedDestination = Invoke-Dispatcher $groupRoot $destinationCase.Path 1
        Assert-True ($rejectedDestination.local_process_ready -eq $false) `
            "$($destinationCase.Name) destination produced readiness."
        Assert-True (-not (Test-Path -LiteralPath $destinationCase.Path)) `
            "$($destinationCase.Name) destination received an artifact."
    }

    foreach ($token in @('ffi.C', 'Start-Process', 'Invoke-Expression', 'cmd.exe', 'powershell.exe -Command')) {
        $sources = (Get-Content -LiteralPath (Join-Path $groupRoot 'lua/live_galaxy_candidate_entry.lua') -Raw) +
            (Get-Content -LiteralPath (Join-Path $groupRoot 'tools/x4-verification/candidate-adapters.psm1') -Raw)
        Assert-True ($sources -notmatch [regex]::Escape($token)) "Generated untrusted source exposes '$token'."
    }
    Write-Output 'candidate-package-adversarial: PASS'
}
finally {
    if (Test-Path -LiteralPath $scratch) { Remove-Item -LiteralPath $scratch -Recurse -Force }
    if ($hasPreparedBuild) { Assert-PreparedBuild $PreparedBuildRoot $PreparedBuildKey }
}
