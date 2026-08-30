param(
    [Parameter(Mandatory)]
    [string]$RequestPath,
    [Parameter(Mandatory)]
    [string]$ResponsePath,
    [Parameter(Mandatory)]
    [int]$DeadlineMs
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$workerPath = Join-Path $root 'tools/x4-verification/isolation/candidate-worker.ps1'
$protocolPath = Join-Path $root 'tools/x4-verification/contracts/candidate-worker-protocol.v1.json'
$protocol = Get-Content -LiteralPath $protocolPath -Raw | ConvertFrom-Json
$validatedResponsePath = $null
$readinessObserved = $false
$readinessObservedAt = $null
$timeoutArmedAt = $null

function Write-Result([string]$Status, [string]$Code, [object]$Response = $null) {
    [ordered]@{
        schema_version = 'candidate-worker-launch.v1'
        status = $Status
        accepted = $Status -eq 'ok'
        diagnostic_code = $Code
        readiness_observed = $script:readinessObserved
        readiness_observed_at_utc = $script:readinessObservedAt
        timeout_armed_at_utc = $script:timeoutArmedAt
        response = $Response
    } | ConvertTo-Json -Depth 10 -Compress
}

function Test-ExactFields([psobject]$Value, [string[]]$Expected) {
    if ($null -eq $Value -or $Value -isnot [pscustomobject]) { return $false }
    $actual = @($Value.PSObject.Properties.Name | Sort-Object)
    $wanted = @($Expected | Sort-Object)
    $difference = @(Compare-Object -ReferenceObject $wanted -DifferenceObject $actual)
    return ($actual.Count -eq $wanted.Count -and $difference.Count -eq 0)
}

function Test-BoundedText([object]$Value, [int]$Maximum) {
    return ($Value -is [string] -and $Value.Length -gt 0 -and $Value.Length -le $Maximum)
}

try {
    if ($DeadlineMs -lt $protocol.bounds.min_deadline_ms -or
        $DeadlineMs -gt $protocol.bounds.max_deadline_ms) {
        Write-Result 'request-rejected' 'deadline-invalid'
        exit 0
    }
    $requestFull = [IO.Path]::GetFullPath($RequestPath)
    $responseFull = [IO.Path]::GetFullPath($ResponsePath)
    if ([IO.Path]::GetDirectoryName($requestFull) -ne [IO.Path]::GetDirectoryName($responseFull) -or
        -not (Test-Path -LiteralPath $requestFull -PathType Leaf) -or
        (Test-Path -LiteralPath $responseFull)) {
        Write-Result 'request-rejected' 'request-path-invalid'
        exit 0
    }
    $validatedResponsePath = $responseFull
    $childPidPath = "$responseFull.child.pid"
    $readinessPath = "$responseFull.child.ready"
    if ((Test-Path -LiteralPath $childPidPath) -or (Test-Path -LiteralPath $readinessPath)) {
        Write-Result 'request-rejected' 'readiness-path-invalid'
        exit 0
    }
    $requestInfo = Get-Item -LiteralPath $requestFull
    if ($requestInfo.Length -gt $protocol.bounds.max_request_bytes) {
        Write-Result 'request-rejected' 'request-bytes-exceeded'
        exit 0
    }
    $request = Get-Content -LiteralPath $requestFull -Raw | ConvertFrom-Json -DateKind String
    if (-not (Test-ExactFields $request $protocol.request_fields) -or
        -not (Test-ExactFields $request.input $protocol.input_fields) -or
        $request.schema_version -ne $protocol.schema_version -or
        $protocol.adapter_allowlist -notcontains $request.adapter_id) {
        Write-Result 'request-rejected' 'request-schema-invalid'
        exit 0
    }
    foreach ($identity in @($request.request_id, $request.run_id, $request.candidate_id, $request.adapter_id)) {
        if (-not (Test-BoundedText $identity $protocol.bounds.max_identity_bytes)) {
            Write-Result 'request-rejected' 'request-identity-invalid'
            exit 0
        }
    }
    if (-not (Test-BoundedText $request.input.expected_result $protocol.bounds.max_result_bytes) -or
        $request.input.fixture -isnot [pscustomobject] -or
        [Text.Encoding]::UTF8.GetByteCount(($request.input.fixture | ConvertTo-Json -Compress -Depth 8)) -gt 2048 -or
        $request.input.max_work_units -isnot [long] -and $request.input.max_work_units -isnot [int] -or
        $request.input.max_work_units -lt 1 -or
        $request.input.max_work_units -gt $protocol.bounds.max_work_units) {
        Write-Result 'request-rejected' 'request-input-invalid'
        exit 0
    }
    $issuedAt = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParse([string]$request.issued_at_utc, [ref]$issuedAt)) {
        Write-Result 'request-rejected' 'request-time-invalid'
        exit 0
    }

    $killTreeMethod = [Diagnostics.Process].GetMethods() | Where-Object {
        $_.Name -eq 'Kill' -and $_.GetParameters().Count -eq 1 -and
        $_.GetParameters()[0].ParameterType -eq [bool]
    } | Select-Object -First 1
    if ($null -eq $killTreeMethod) {
        Write-Result 'isolation-platform-unsupported' 'ISOLATION_PLATFORM_UNSUPPORTED'
        exit 0
    }

    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = (Get-Process -Id $PID).Path
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    if ($IsWindows) { $startInfo.WindowStyle = [Diagnostics.ProcessWindowStyle]::Hidden }
    $startInfo.ArgumentList.Add('-NoProfile')
    $startInfo.ArgumentList.Add('-NonInteractive')
    $startInfo.ArgumentList.Add('-File')
    $startInfo.ArgumentList.Add($workerPath)
    $startInfo.ArgumentList.Add('-RequestPath')
    $startInfo.ArgumentList.Add($requestFull)
    $startInfo.ArgumentList.Add('-ResponsePath')
    $startInfo.ArgumentList.Add($responseFull)

    $startedAt = [DateTimeOffset]::UtcNow
    $process = [Diagnostics.Process]::Start($startInfo)
    if ($request.adapter_id -eq 'local-contract-child-endless') {
        $readinessDeadline = $startedAt.AddMilliseconds([Math]::Min(2000, [int]$protocol.bounds.max_deadline_ms))
        while (-not (Test-Path -LiteralPath $readinessPath -PathType Leaf) -and
            -not $process.HasExited -and [DateTimeOffset]::UtcNow -lt $readinessDeadline) {
            Start-Sleep -Milliseconds 10
            $process.Refresh()
        }
        if (-not (Test-Path -LiteralPath $readinessPath -PathType Leaf) -or
            -not (Test-Path -LiteralPath $childPidPath -PathType Leaf)) {
            if (-not $process.HasExited) {
                $process.Kill($true)
                [void]$process.WaitForExit($protocol.bounds.kill_wait_ms)
            }
            Write-Result 'worker-response-rejected' 'worker-readiness-missing'
            exit 0
        }
        $pidInfo = Get-Item -LiteralPath $childPidPath
        $readinessInfo = Get-Item -LiteralPath $readinessPath
        if (($pidInfo.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
            ($readinessInfo.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
            $pidInfo.Length -lt 1 -or $pidInfo.Length -gt 16 -or
            $readinessInfo.Length -lt 1 -or $readinessInfo.Length -gt 512) {
            $process.Kill($true)
            [void]$process.WaitForExit($protocol.bounds.kill_wait_ms)
            Write-Result 'worker-response-rejected' 'worker-readiness-invalid'
            exit 0
        }
        $pidText = [IO.File]::ReadAllText($pidInfo.FullName, [Text.UTF8Encoding]::new($false))
        $readinessText = [IO.File]::ReadAllText($readinessInfo.FullName, [Text.UTF8Encoding]::new($false))
        try { $readiness = $readinessText | ConvertFrom-Json -DateKind String }
        catch { $readiness = $null }
        $childPid = 0
        $readyAt = [DateTimeOffset]::MinValue
        $readinessFields = @('schema_version', 'request_id', 'child_pid', 'ready_at_utc')
        if ($null -eq $readiness -or -not (Test-ExactFields $readiness $readinessFields) -or
            $readinessText -cne ($readiness | ConvertTo-Json -Compress) -or
            $readiness.schema_version -ne 'candidate-worker-readiness.v1' -or
            $readiness.request_id -ne $request.request_id -or
            -not [int]::TryParse($pidText, [ref]$childPid) -or
            $readiness.child_pid -ne $childPid -or
            -not [DateTimeOffset]::TryParse([string]$readiness.ready_at_utc, [ref]$readyAt) -or
            $readyAt -lt $startedAt.AddSeconds(-1) -or $readyAt -gt [DateTimeOffset]::UtcNow.AddSeconds(1) -or
            $null -eq (Get-Process -Id $childPid -ErrorAction SilentlyContinue)) {
            $process.Kill($true)
            [void]$process.WaitForExit($protocol.bounds.kill_wait_ms)
            Write-Result 'worker-response-rejected' 'worker-readiness-invalid'
            exit 0
        }
        $readinessObserved = $true
        $readinessObservedAt = [DateTimeOffset]::UtcNow.ToString('O')
    }
    $timeoutArmedAt = [DateTimeOffset]::UtcNow.ToString('O')
    if (-not $process.WaitForExit($DeadlineMs)) {
        $process.Kill($true)
        [void]$process.WaitForExit($protocol.bounds.kill_wait_ms)
        if (Test-Path -LiteralPath $responseFull) { Remove-Item -LiteralPath $responseFull -Force }
        Write-Result 'worker-timeout' 'worker-timeout'
        exit 0
    }
    if ($process.ExitCode -ne 0 -or -not (Test-Path -LiteralPath $responseFull -PathType Leaf)) {
        Write-Result 'worker-response-rejected' 'worker-output-missing'
        exit 0
    }
    $responseInfo = Get-Item -LiteralPath $responseFull
    if ($responseInfo.Length -gt $protocol.bounds.max_response_bytes) {
        Remove-Item -LiteralPath $responseFull -Force
        Write-Result 'worker-response-rejected' 'worker-output-bytes-exceeded'
        exit 0
    }
    $responseText = Get-Content -LiteralPath $responseFull -Raw
    try { $response = $responseText | ConvertFrom-Json -DateKind String }
    catch {
        Remove-Item -LiteralPath $responseFull -Force
        Write-Result 'worker-response-rejected' 'worker-output-malformed'
        exit 0
    }
    $canonicalResponse = $response | ConvertTo-Json -Depth 10 -Compress
    if ($responseText -cne $canonicalResponse) {
        Remove-Item -LiteralPath $responseFull -Force
        Write-Result 'worker-response-rejected' 'worker-output-noncanonical'
        exit 0
    }
    if (-not (Test-ExactFields $response $protocol.response_fields)) {
        Remove-Item -LiteralPath $responseFull -Force
        Write-Result 'worker-response-rejected' 'worker-output-schema-invalid'
        exit 0
    }
    if ($response.schema_version -ne $protocol.schema_version -or
        $response.request_id -ne $request.request_id -or $response.run_id -ne $request.run_id -or
        $response.candidate_id -ne $request.candidate_id -or $response.adapter_id -ne $request.adapter_id) {
        Remove-Item -LiteralPath $responseFull -Force
        Write-Result 'worker-response-rejected' 'worker-output-identity-mismatch'
        exit 0
    }
    $producedAt = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParse([string]$response.produced_at_utc, [ref]$producedAt) -or
        $producedAt -lt $startedAt.AddSeconds(-1) -or $producedAt -gt [DateTimeOffset]::UtcNow.AddSeconds(1)) {
        Remove-Item -LiteralPath $responseFull -Force
        Write-Result 'worker-response-rejected' 'worker-output-stale'
        exit 0
    }
    if ($response.status -ne 'completed' -or $response.completeness -ne 'complete' -or
        -not (Test-BoundedText $response.actual_result $protocol.bounds.max_result_bytes) -or
        $response.actual_result -ne $request.input.expected_result -or
        $response.work_units -lt 0 -or $response.work_units -gt $request.input.max_work_units -or
        @($response.observations).Count -gt $protocol.bounds.max_observations) {
        Remove-Item -LiteralPath $responseFull -Force
        Write-Result 'worker-response-rejected' 'worker-output-contract-invalid'
        exit 0
    }
    Write-Result 'ok' 'none' $response
}
catch {
    if ($null -ne $validatedResponsePath -and (Test-Path -LiteralPath $validatedResponsePath)) {
        Remove-Item -LiteralPath $validatedResponsePath -Force
    }
    Write-Result 'worker-response-rejected' 'worker-internal-failure'
}
