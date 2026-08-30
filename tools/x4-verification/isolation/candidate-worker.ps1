param(
    [Parameter(Mandatory)]
    [string]$RequestPath,
    [Parameter(Mandatory)]
    [string]$ResponsePath
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$protocolPath = Join-Path $root 'tools/x4-verification/contracts/candidate-worker-protocol.v1.json'
$adapterModulePath = Join-Path $root 'tools/x4-verification/candidate-adapters.psm1'
$boundedReaderPath = Join-Path $root 'tools/x4-verification/bounded-file.psm1'
Import-Module $boundedReaderPath -Force
$protocolRead = Read-BoundedFile $protocolPath 32768 'worker-protocol-invalid'
$protocol = [Text.Encoding]::UTF8.GetString($protocolRead.Bytes) | ConvertFrom-Json

function Test-ExactFields([psobject]$Value, [string[]]$Expected) {
    if ($null -eq $Value -or $Value -isnot [pscustomobject]) { return $false }
    $actual = @($Value.PSObject.Properties.Name | Sort-Object)
    $wanted = @($Expected | Sort-Object)
    $difference = @(Compare-Object -ReferenceObject $wanted -DifferenceObject $actual)
    return ($actual.Count -eq $wanted.Count -and $difference.Count -eq 0)
}

function Write-WorkerText([string]$Text, [bool]$Atomic = $true) {
    if ($Atomic) {
        $temporaryPath = "$ResponsePath.$PID.tmp"
        [IO.File]::WriteAllText($temporaryPath, $Text, [Text.UTF8Encoding]::new($false))
        Move-Item -LiteralPath $temporaryPath -Destination $ResponsePath
    }
    else {
        [IO.File]::WriteAllText($ResponsePath, $Text, [Text.UTF8Encoding]::new($false))
    }
}

function Write-AtomicSidecar([string]$Path, [string]$Text) {
    $temporaryPath = "$Path.$PID.tmp"
    [IO.File]::WriteAllText($temporaryPath, $Text, [Text.UTF8Encoding]::new($false))
    Move-Item -LiteralPath $temporaryPath -Destination $Path
}

try {
    $requestRead = Read-BoundedFile $RequestPath $protocol.bounds.max_request_bytes `
        'request-path-invalid' 'request-bytes-exceeded' 'request-identity-changed'
}
catch { exit 20 }
$request = [Text.Encoding]::UTF8.GetString($requestRead.Bytes) | ConvertFrom-Json -DateKind String
if (-not (Test-ExactFields $request $protocol.request_fields)) { exit 21 }
if (-not (Test-ExactFields $request.input $protocol.input_fields)) { exit 22 }
if ($request.schema_version -ne $protocol.schema_version) { exit 23 }
if ($protocol.adapter_allowlist -notcontains $request.adapter_id) { exit 24 }

if ($request.adapter_id -eq 'local-contract-endless') {
    while ($true) { Start-Sleep -Seconds 1 }
}
if ($request.adapter_id -eq 'local-contract-child-endless') {
    $childStart = [Diagnostics.ProcessStartInfo]::new()
    $childStart.FileName = (Get-Process -Id $PID).Path
    $childStart.UseShellExecute = $false
    $childStart.CreateNoWindow = $true
    if ($IsWindows) { $childStart.WindowStyle = [Diagnostics.ProcessWindowStyle]::Hidden }
    $childStart.ArgumentList.Add('-NoProfile')
    $childStart.ArgumentList.Add('-NonInteractive')
    $childStart.ArgumentList.Add('-Command')
    $childStart.ArgumentList.Add('Start-Sleep -Seconds 120')
    $child = [Diagnostics.Process]::Start($childStart)
    $child.Refresh()
    if ($child.HasExited) { exit 26 }
    $pidPath = "$ResponsePath.child.pid"
    $readinessPath = "$ResponsePath.child.ready"
    Write-AtomicSidecar $pidPath $child.Id.ToString()
    $readiness = [ordered]@{
        schema_version = 'candidate-worker-readiness.v1'
        request_id = $request.request_id
        child_pid = $child.Id
        ready_at_utc = [DateTimeOffset]::UtcNow.ToString('O')
    } | ConvertTo-Json -Compress
    Write-AtomicSidecar $readinessPath $readiness
    while ($true) { Start-Sleep -Seconds 1 }
}
if ($request.adapter_id -eq 'local-contract-late-success') {
    Start-Sleep -Seconds 1
}
if ($request.adapter_id -eq 'local-contract-malformed') {
    Write-WorkerText '{not-json' $false
    exit 0
}
if ($request.adapter_id -eq 'local-contract-partial') {
    Write-WorkerText '{"schema_version":"candidate-worker.v1"' $false
    exit 0
}

$response = [ordered]@{
    schema_version = $protocol.schema_version
    request_id = $request.request_id
    run_id = $request.run_id
    candidate_id = $request.candidate_id
    adapter_id = $request.adapter_id
    produced_at_utc = [DateTimeOffset]::UtcNow.ToString('O')
    status = 'completed'
    actual_result = $request.input.expected_result
    completeness = 'complete'
    work_units = 1
    observations = @('local-contract-observation')
    elapsed_real_ms = 1
    elapsed_game_ms = 0
    seta_state = 'not_applicable'
    diagnostic_code = 'none'
}

$candidateAdapterIds = @(
    'p051-cadence-seta',
    'p051-lifecycle-reload',
    'p051-mod-stack-compatibility',
    'p051-native-count-fill-runtime',
    'p051-native-fill-completeness',
    'p051-native-identity-closure',
    'p051-native-volume-envelope'
)
if ($candidateAdapterIds -ccontains [string]$request.candidate_id) {
    if ($request.adapter_id -ne 'local-contract-success') { exit 25 }
    Import-Module $adapterModulePath -Force
    $adapterResult = Invoke-CandidateAdapter `
        -CandidateId $request.candidate_id `
        -Fixture $request.input.fixture `
        -MaxWorkUnits $request.input.max_work_units
    $response.status = $adapterResult.status
    $response.actual_result = $adapterResult.actual_result
    $response.completeness = $adapterResult.completeness
    $response.work_units = $adapterResult.work_units
    $response.observations = @($adapterResult.observations)
    $response.diagnostic_code = $adapterResult.diagnostic_code
}

switch ($request.adapter_id) {
    'local-contract-identity-swap' { $response.run_id = 'forged-run' }
    'local-contract-adapter-swap' { $response.adapter_id = 'local-contract-success' }
    'local-contract-stale' { $response.produced_at_utc = '2000-01-01T00:00:00.0000000Z' }
    'local-contract-extra-field' { $response.extra_authority = 'forged' }
    'local-contract-oversized' { $response.diagnostic_code = 'x' * 9000 }
}

$json = $response | ConvertTo-Json -Depth 8 -Compress
if ($request.adapter_id -eq 'local-contract-noncanonical') {
    $json = $response | ConvertTo-Json -Depth 8
}
if ($request.adapter_id -eq 'local-contract-duplicate') {
    $json = "[$json,$json]"
}
Write-WorkerText $json
