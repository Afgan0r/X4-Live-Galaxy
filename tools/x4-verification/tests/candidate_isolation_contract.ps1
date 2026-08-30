param(
    [ValidateSet('all', 'success', 'timeout', 'responses')]
    [string]$Case = 'all'
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$launcher = Join-Path $root 'tools/x4-verification/isolation/invoke-candidate-worker.ps1'
$protocolPath = Join-Path $root 'tools/x4-verification/contracts/candidate-worker-protocol.v1.json'

function Assert-Contract([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function New-Request([string]$AdapterId) {
    return [ordered]@{
        schema_version = 'candidate-worker.v1'
        request_id = "request-$AdapterId"
        run_id = 'run-isolation-contract'
        candidate_id = 'candidate-isolation-contract'
        adapter_id = $AdapterId
        issued_at_utc = [DateTimeOffset]::UtcNow.ToString('O')
        input = [ordered]@{
            expected_result = 'local-contract-valid'
            fixture = [ordered]@{}
            max_work_units = 8
        }
    }
}

function Invoke-Isolated(
    [string]$Workspace,
    [string]$AdapterId,
    [int]$DeadlineMs = 1500
) {
    $requestPath = Join-Path $Workspace "$AdapterId.request.json"
    $responsePath = Join-Path $Workspace "$AdapterId.response.json"
    (New-Request $AdapterId) | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $requestPath -Encoding utf8NoBOM
    $output = @(& pwsh -NoProfile -File $launcher -RequestPath $requestPath `
        -ResponsePath $responsePath -DeadlineMs $DeadlineMs 2>&1)
    Assert-Contract ($LASTEXITCODE -eq 0) "Launcher failed for $AdapterId`: $($output -join ' ')"
    Assert-Contract ($output.Count -eq 1) "Launcher emitted non-canonical output for $AdapterId."
    $result = $output[0].ToString() | ConvertFrom-Json
    return [pscustomobject]@{
        Result = $result
        ResponsePath = $responsePath
        ChildPidPath = "$responsePath.child.pid"
    }
}

Assert-Contract (Test-Path -LiteralPath $protocolPath -PathType Leaf) 'Worker protocol contract is missing.'
$protocol = Get-Content -LiteralPath $protocolPath -Raw | ConvertFrom-Json
Assert-Contract ($protocol.schema_version -eq 'candidate-worker.v1') 'Worker protocol version drifted.'
Assert-Contract ($protocol.bounds.max_request_bytes -le 4096) 'Request byte bound is too broad.'
Assert-Contract ($protocol.bounds.max_response_bytes -le 8192) 'Response byte bound is too broad.'
Assert-Contract ($protocol.native_modes -contains 'forbidden') 'Direct native mode is not explicitly forbidden.'
Assert-Contract (@($protocol.native_modes).Count -eq 1) 'A native or in-process mode was admitted.'

$workspaceRoot = Join-Path $root 'tools/.cache/candidate-isolation-contract'
$workspace = Join-Path $workspaceRoot ([Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $workspace -Force | Out-Null

try {
    if ($Case -in @('all', 'success')) {
        $run = Invoke-Isolated $workspace 'local-contract-success'
        Assert-Contract ($run.Result.status -eq 'ok' -and $run.Result.accepted -eq $true) 'Success was not accepted.'
        Assert-Contract ($run.Result.response.adapter_id -eq 'local-contract-success') 'Success adapter identity drifted.'
        Assert-Contract ($run.Result.response.actual_result -eq 'local-contract-valid') 'Success result drifted.'
        Assert-Contract ($run.Result.response.completeness -eq 'complete') 'Success completeness drifted.'
        Write-Output 'PASS candidate isolation success'
    }

    if ($Case -in @('all', 'timeout')) {
        $run = Invoke-Isolated $workspace 'local-contract-endless' 250
        Assert-Contract ($run.Result.status -eq 'worker-timeout' -and $run.Result.accepted -eq $false) 'Endless worker did not time out.'
        Assert-Contract (-not (Test-Path -LiteralPath $run.ResponsePath)) 'Timed-out worker left an accepted response.'

        $run = Invoke-Isolated $workspace 'local-contract-child-endless' 500
        Assert-Contract ($run.Result.status -eq 'worker-timeout' -and $run.Result.accepted -eq $false) 'Child worker did not time out.'
        Assert-Contract (Test-Path -LiteralPath $run.ChildPidPath -PathType Leaf) 'Child PID evidence is missing.'
        $childPid = [int](Get-Content -LiteralPath $run.ChildPidPath -Raw)
        Start-Sleep -Milliseconds 100
        Assert-Contract ($null -eq (Get-Process -Id $childPid -ErrorAction SilentlyContinue)) 'Descendant process survived timeout.'
        Assert-Contract (-not (Test-Path -LiteralPath $run.ResponsePath)) 'Child timeout left an accepted response.'
        Write-Output 'PASS candidate isolation process-tree timeout'
    }

    if ($Case -in @('all', 'responses')) {
        $rejections = @(
            'local-contract-malformed',
            'local-contract-identity-swap',
            'local-contract-adapter-swap',
            'local-contract-oversized',
            'local-contract-partial',
            'local-contract-stale',
            'local-contract-extra-field',
            'local-contract-noncanonical',
            'local-contract-duplicate'
        )
        foreach ($adapterId in $rejections) {
            $run = Invoke-Isolated $workspace $adapterId
            Assert-Contract ($run.Result.status -eq 'worker-response-rejected') "$adapterId was not rejected."
            Assert-Contract ($run.Result.accepted -eq $false) "$adapterId became accepted."
        }
        $run = Invoke-Isolated $workspace 'local-contract-late-success' 250
        Assert-Contract ($run.Result.status -eq 'worker-timeout' -and $run.Result.accepted -eq $false) 'Late success won the timeout race.'
        Start-Sleep -Milliseconds 400
        Assert-Contract (-not (Test-Path -LiteralPath $run.ResponsePath)) 'Post-timeout response became visible.'

        foreach ($field in @('command', 'module', 'executable', 'native_mode')) {
            $requestPath = Join-Path $workspace "authority-$field.request.json"
            $responsePath = Join-Path $workspace "authority-$field.response.json"
            $request = New-Request 'local-contract-success'
            $request[$field] = 'forged-authority'
            $request | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $requestPath -Encoding utf8NoBOM
            $output = @(& pwsh -NoProfile -File $launcher -RequestPath $requestPath `
                -ResponsePath $responsePath -DeadlineMs 1500 2>&1)
            Assert-Contract ($LASTEXITCODE -eq 0 -and $output.Count -eq 1) "$field injection broke canonical output."
            $result = $output[0].ToString() | ConvertFrom-Json
            Assert-Contract ($result.status -eq 'request-rejected' -and $result.accepted -eq $false) "$field injection was accepted."
        }

        $requestPath = Join-Path $workspace 'forged-success.request.json'
        $responsePath = Join-Path $workspace 'forged-success.response.json'
        (New-Request 'local-contract-success') | ConvertTo-Json -Depth 8 |
            Set-Content -LiteralPath $requestPath -Encoding utf8NoBOM
        '{"status":"completed","actual_result":"forged"}' |
            Set-Content -LiteralPath $responsePath -Encoding utf8NoBOM
        $output = @(& pwsh -NoProfile -File $launcher -RequestPath $requestPath `
            -ResponsePath $responsePath -DeadlineMs 1500 2>&1)
        Assert-Contract ($LASTEXITCODE -eq 0 -and $output.Count -eq 1) 'Forged response broke canonical output.'
        $result = $output[0].ToString() | ConvertFrom-Json
        Assert-Contract ($result.status -eq 'request-rejected' -and $result.accepted -eq $false) 'Forged preexisting success was accepted.'

        $launcherSource = Get-Content -LiteralPath $launcher -Raw
        $workerSource = Get-Content -LiteralPath (Join-Path $root 'tools/x4-verification/isolation/candidate-worker.ps1') -Raw
        Assert-Contract ($launcherSource -notmatch 'Invoke-Expression') 'Launcher admits dynamic command execution.'
        Assert-Contract ($workerSource -notmatch 'Invoke-Expression') 'Worker admits dynamic command execution.'
        Assert-Contract ($workerSource -notmatch 'ffi\.C') 'Worker contains a direct native binding.'
        Write-Output 'PASS candidate isolation response rejection'
    }
}
finally {
    if (Test-Path -LiteralPath $workspace) {
        Remove-Item -LiteralPath $workspace -Recurse -Force
    }
}
