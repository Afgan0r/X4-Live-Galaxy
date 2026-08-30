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

function Invoke-Dispatcher([string]$GroupRoot, [string]$OutputPath, [int]$ExpectedExit) {
    $text = & pwsh -NoProfile -File $dispatcherPath -GroupRoot $GroupRoot -OutputPath $OutputPath 2>&1 | Out-String
    Assert-True ($LASTEXITCODE -eq $ExpectedExit) "Dispatcher exit $LASTEXITCODE, expected $ExpectedExit`: $text"
    return $text.Trim() | ConvertFrom-Json -DateKind String
}

$parameters = (Get-Command $dispatcherPath).Parameters.Keys
foreach ($forbidden in @('RootPath', 'TrustRootPath', 'CertificatePath', 'KeyName', 'TestMode', 'Command', 'ModulePath', 'WorkerPath')) {
    Assert-True ($parameters -notcontains $forbidden) "Production dispatcher exposes forbidden selector '$forbidden'."
}

$scratch = Join-Path ([IO.Path]::GetTempPath()) ('live-galaxy-plan08-adversarial-' + [guid]::NewGuid().ToString('N'))
$buildRoot = Join-Path $scratch 'builds'
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
    $builderText = & pwsh -NoProfile -File $builderPath -BuildRoot $buildRoot -MatrixPath $matrixPath 2>&1 | Out-String
    Assert-True ($LASTEXITCODE -eq 0) "Builder failed: $builderText"
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
            Assert-True ($swapResult.reason_code -in @('COMPONENT_DIGEST_MISMATCH', 'SNAPSHOT_DIGEST_MISMATCH')) `
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
}
