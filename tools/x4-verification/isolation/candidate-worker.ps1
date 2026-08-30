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
$protocol = Get-Content -LiteralPath $protocolPath -Raw | ConvertFrom-Json

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

$requestInfo = Get-Item -LiteralPath $RequestPath -ErrorAction Stop
if ($requestInfo.Length -gt $protocol.bounds.max_request_bytes) { exit 20 }
$request = Get-Content -LiteralPath $RequestPath -Raw | ConvertFrom-Json
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
    [IO.File]::WriteAllText("$ResponsePath.child.pid", $child.Id.ToString(), [Text.UTF8Encoding]::new($false))
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

switch ($request.adapter_id) {
    'local-contract-identity-swap' { $response.run_id = 'forged-run' }
    'local-contract-adapter-swap' { $response.adapter_id = 'local-contract-success' }
    'local-contract-stale' { $response.produced_at_utc = '2000-01-01T00:00:00.0000000Z' }
    'local-contract-extra-field' { $response.extra_authority = 'forged' }
    'local-contract-oversized' { $response.diagnostic_code = 'x' * 9000 }
}

$json = $response | ConvertTo-Json -Depth 8 -Compress
if ($request.adapter_id -eq 'local-contract-duplicate') {
    $json = "[$json,$json]"
}
Write-WorkerText $json
