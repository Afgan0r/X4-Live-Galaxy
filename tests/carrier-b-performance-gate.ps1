function Get-HeavyFrameBudgetMillis([hashtable]$Policy) {
    $target = [double]$Policy.target_fps
    if ($target -le 0) { throw 'PERFORMANCE_TARGET_FPS_INVALID' }
    1000 / $target
}

function Assert-HeavyPerformanceMeasurement([hashtable]$Policy, [hashtable]$Measurement) {
    if ([int]$Policy.schema_version -ne 1) { throw 'PERFORMANCE_POLICY_VERSION_UNSUPPORTED' }
    $frameBudget = Get-HeavyFrameBudgetMillis $Policy
    if ([double]$Measurement.callback_p95_millis -gt $frameBudget) {
        throw "CALLBACK_P95_FRAME_BUDGET_EXCEEDED:target_fps=$($Policy.target_fps)"
    }
    if ([double]$Measurement.callback_max_millis -gt $frameBudget) {
        throw "FRAME_BUDGET_EXCEEDED:callback:target_fps=$($Policy.target_fps)"
    }
    foreach ($entry in $Measurement.stage_max_millis.GetEnumerator()) {
        if ([double]$entry.Value -gt $frameBudget) {
            throw "FRAME_BUDGET_EXCEEDED:$($entry.Key):target_fps=$($Policy.target_fps)"
        }
    }
    foreach ($stage in @($Policy.required_stages)) {
        if (-not $Measurement.stage_max_millis.ContainsKey($stage)) {
            throw "PERFORMANCE_STAGE_MISSING:$stage"
        }
    }
    $minimumThroughput = [double]$Policy.baseline_records_per_callback *
        (1 - [double]$Policy.maximum_callback_work_regression_percent / 100)
    if ([double]$Measurement.records_per_callback -lt $minimumThroughput) {
        throw "THROUGHPUT_REGRESSION:minimum_records_per_callback=$minimumThroughput"
    }
    if ([int]$Measurement.final_backlog -ne [int]$Policy.required_final_backlog) {
        throw "FINAL_BACKLOG:expected=$($Policy.required_final_backlog)"
    }
}

function Read-HeavyPerformanceMeasurement([string]$Path) {
    $values = @{}
    $stages = @{}
    foreach ($line in Get-Content -LiteralPath $Path) {
        if ($line -match '^([a-z0-9_]+)=([0-9]+(?:\.[0-9]+)?)$') {
            $values[$Matches[1]] = [double]::Parse($Matches[2], [Globalization.CultureInfo]::InvariantCulture)
        } elseif ($line -match '^stage=(\S+).* max_millis=([0-9]+(?:\.[0-9]+)?)$') {
            $stages[$Matches[1]] = [double]::Parse($Matches[2], [Globalization.CultureInfo]::InvariantCulture)
        }
    }
    foreach ($required in @('committed', 'synthetic_core_records', 'elapsed_millis',
        'callback_samples', 'callback_p95_millis', 'callback_max_millis', 'final_backlog')) {
        if (-not $values.ContainsKey($required)) { throw "PERFORMANCE_METRIC_MISSING:$required" }
    }
    @{
        callback_p95_millis = $values.callback_p95_millis
        callback_max_millis = $values.callback_max_millis
        records_per_second = $values.committed * $values.synthetic_core_records * 1000 / $values.elapsed_millis
        records_per_callback = $values.committed * $values.synthetic_core_records / $values.callback_samples
        final_backlog = $values.final_backlog
        stage_max_millis = $stages
    }
}

function Format-HeavyPerformanceGate([hashtable]$Policy, [hashtable]$Measurement) {
    $frameBudget = Get-HeavyFrameBudgetMillis $Policy
    'PERFORMANCE_GATE target_fps={0} frame_budget_millis={1:R} callback_budget_millis={2} callback_p95_millis={3:R} callback_max_millis={4:R} records_per_second={5:R} records_per_callback={6:R} final_backlog={7}' -f
        $Policy.target_fps, $frameBudget, $Policy.callback_budget_millis,
        $Measurement.callback_p95_millis, $Measurement.callback_max_millis,
        $Measurement.records_per_second, $Measurement.records_per_callback,
        $Measurement.final_backlog
}
