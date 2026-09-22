$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'carrier-b-performance-gate.ps1')

$policy = @{
    schema_version = 1
    target_fps = 144
    callback_budget_millis = 2
    baseline_records_per_callback = 1
    maximum_callback_work_regression_percent = 20
    required_final_backlog = 0
    required_stages = @('source_read_core', 'push_record')
}
$passing = @{
    callback_p95_millis = 2
    callback_max_millis = 6
    records_per_callback = 0.9
    final_backlog = 0
    stage_max_millis = @{ source_read_core = 6; push_record = 1 }
}

$budget = Get-HeavyFrameBudgetMillis $policy
if ([Math]::Abs($budget - (1000 / 144)) -gt 0.000001) {
    throw 'frame budget was not derived from target_fps'
}
Assert-HeavyPerformanceMeasurement $policy $passing

foreach ($case in @(
    @{ Name = 'callback_p95'; Change = { param($m) $m.callback_p95_millis = 7 }; Error = 'CALLBACK_P95_FRAME_BUDGET_EXCEEDED' },
    @{ Name = 'callback_max'; Change = { param($m) $m.callback_max_millis = 7 }; Error = 'FRAME_BUDGET_EXCEEDED' },
    @{ Name = 'stage_max'; Change = { param($m) $m.stage_max_millis.source_read_core = 7 }; Error = 'FRAME_BUDGET_EXCEEDED' },
    @{ Name = 'throughput'; Change = { param($m) $m.records_per_callback = 0.79 }; Error = 'THROUGHPUT_REGRESSION' },
    @{ Name = 'backlog'; Change = { param($m) $m.final_backlog = 1 }; Error = 'FINAL_BACKLOG' }
    @{ Name = 'stage_missing'; Change = { param($m) $m.stage_max_millis.Remove('source_read_core') }; Error = 'PERFORMANCE_STAGE_MISSING' }
)) {
    $measurement = @{
        callback_p95_millis = $passing.callback_p95_millis
        callback_max_millis = $passing.callback_max_millis
        records_per_callback = $passing.records_per_callback
        final_backlog = $passing.final_backlog
        stage_max_millis = @{
            source_read_core = $passing.stage_max_millis.source_read_core
            push_record = $passing.stage_max_millis.push_record
        }
    }
    & $case.Change $measurement
    try {
        Assert-HeavyPerformanceMeasurement $policy $measurement
        throw "missing rejection for $($case.Name)"
    } catch {
        if ($_.Exception.Message -notmatch $case.Error) { throw }
    }
}

$fixture = [IO.Path]::GetTempFileName()
try {
    [IO.File]::WriteAllLines($fixture, @(
        'committed=4', 'synthetic_core_records=129', 'elapsed_millis=1000',
        'callback_samples=100', 'callback_p95_millis=3.1', 'callback_max_millis=4.2',
        'final_backlog=0', 'stage=source_read_core samples=10 p95_millis=1 max_millis=2',
        'stage=push_record samples=10 p95_millis=1 max_millis=2'
    ))
    $parsed = Read-HeavyPerformanceMeasurement $fixture
    if ($parsed.callback_p95_millis -ne 3.1 -or $parsed.records_per_callback -ne 5.16) {
        throw 'metric parser lost a numeric field name or callback-normalized throughput'
    }
} finally { Remove-Item -LiteralPath $fixture -Force }

Write-Output 'PASS target_fps_derivation=true callback_and_stage_frame_budget=true throughput_and_backlog=true'
