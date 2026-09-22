function Assert-HeavySamePeerInterleave([string]$Data) {
    $events = @(Get-ChildItem -LiteralPath $Data -File | Where-Object { $_.Name -like 'operational-history.jsonl*' } |
        ForEach-Object { Get-Content -LiteralPath $_.FullName | ForEach-Object { $_ | ConvertFrom-Json } })
    $first = $events | Where-Object { $_.state -ceq 'committed' -and $_.revision -eq 1 } | Select-Object -First 1
    $last = $events | Where-Object { $_.state -ceq 'committed' -and $_.revision -eq 20 } | Select-Object -First 1
    if ($null -eq $first -or $null -eq $last) { throw 'THROUGHPUT_SAME_PEER_HISTORY_MISSING' }
    $active = @($events | Where-Object { $_.at -ge $first.at -and $_.at -le $last.at })
    if (@($active | Where-Object { $_.state -ceq 'disconnected' -or $_.reason -cin @('peer-disconnected', 'peer-absent', 'compatible-session') }).Count -gt 0) {
        throw 'THROUGHPUT_SAME_PEER_CONTINUITY_LOSS'
    }
    if (@($active | Where-Object { $_.state -ceq 'degraded' -and $_.reason -ceq 'receive-timeout' }).Count -ne 1 -or
        @($active | Where-Object { $_.state -ceq 'recovered' -and $_.reason -ceq 'stale-scope-rotated' }).Count -ne 1 -or
        @($active | Where-Object { $_.state -ceq 'rejected' -and $_.reason -ceq 'receive-timeout' }).Count -ne 0 -or
        @($active.session | Sort-Object -Unique).Count -ne 1 -or @($active.epoch | Sort-Object -Unique).Count -ne 1) {
        throw 'THROUGHPUT_SAME_PEER_REFRESH_MISSING'
    }
    Write-Output 'SAME_PEER_INTERLEAVE revisions=1..20 inactivity_recovery=recovered sessions=1 epochs=1 reconnect=false'
}

function Invoke-HeavyConfiguredRestart([string]$Run, [string]$HostExecutable, [string]$Data, [string]$Limits, [string]$Repo, [switch]$Throughput, [switch]$Interleave, [switch]$PerformanceGate) {
    . (Join-Path $Repo 'tests/carrier-b-heavy-oracles.ps1')
    . (Join-Path $Repo 'tests/carrier-b-performance-gate.ps1')
    if ($Interleave -and -not $Throughput) { throw 'INTERLEAVE_REQUIRES_LOCAL_THROUGHPUT' }
    $bridge = $null; $producer = $null
    $result = Join-Path $Run 'heavy-result.txt'
    $script = Join-Path $Repo 'extensions/live_galaxy/tests/carrier_b_heavy_configured.lua'
    $readbackRows = @()
    try {
        $bridge = Start-Owned (Join-Path $Repo 'target/release/x4-bridge.exe') @('--data-dir', $Data, '--limits-file', $Limits, '--ship-faction', 'argon') $Run
        $modes = if ($Interleave) { @('first') } else { @('first', 'restart') }
        foreach ($mode in $modes) {
            $marker = if ($Interleave) { 'throughput-interleave' } elseif ($PerformanceGate) { 'performance' } elseif ($Throughput) { 'throughput' } else { '' }
            $producer = Start-Owned $HostExecutable @($Run, $script, $result, $marker, $mode) $Run $true
            $deadline = [DateTime]::UtcNow.AddMilliseconds((Get-Content -LiteralPath $Limits -Raw | ConvertFrom-Json).admission_window_millis)
            while (-not $producer.HasExited -and [DateTime]::UtcNow -lt $deadline) { Start-Sleep -Milliseconds 10; $producer.Refresh() }
            if (-not $producer.HasExited) { throw 'HEAVY_CONFIGURED_HOST_WATCHDOG' }
            $producer.WaitForExit()
            if ($producer.ExitCode -ne 0) { throw "HEAVY_CONFIGURED_HOST_FAILED:$($producer.StandardError.ReadToEnd())" }
            $expected = if ($Interleave) { (1..20) -join ',' } elseif ($mode -ceq 'first') { (1..9) -join ',' } else { (10..14) -join ',' }
            if (-not (Select-String -LiteralPath $result -SimpleMatch "revisions=$expected" -Quiet)) { throw 'HEAVY_CONFIGURED_REVISION_MISMATCH' }
            if ($Throughput) {
                Get-Content -LiteralPath $result
                foreach ($line in Get-Content -LiteralPath $result) {
                    if ($line -match '^capture section=(\S+) revision=(\d+) ') { $readbackRows += ,@($Matches[1], [int]$Matches[2]) }
                }
            }
            if ($PerformanceGate) {
                $policy = Get-Content -LiteralPath (Join-Path $Repo 'config/heavy-ship-performance.json') -Raw |
                    ConvertFrom-Json -AsHashtable
                $activeLimits = Get-Content -LiteralPath $Limits -Raw | ConvertFrom-Json
                if ([double]$policy.callback_budget_millis -ne [double]$activeLimits.callback_budget_millis) {
                    throw 'PERFORMANCE_CALLBACK_PROFILE_MISMATCH'
                }
                $measurement = Read-HeavyPerformanceMeasurement $result
                Assert-HeavyPerformanceMeasurement $policy $measurement
                Write-Output (Format-HeavyPerformanceGate $policy $measurement)
            }
        }
        if (-not $Throughput) { $readbackRows = @(@('ship_core:argon', 2), @('ship_core:argon', 11), @('ship_cargo:argon:g0', 3), @('ship_cargo:argon:g0', 12), @('ship_crew:argon:g0', 4), @('ship_crew:argon:g0', 13), @('ship_loadout:argon:g0', 5), @('ship_loadout:argon:g0', 14)) }
        if ($Interleave -and @($readbackRows | Where-Object { $_[0] -ceq 'ship_core:argon' }).Count -lt 2) { throw 'THROUGHPUT_PARENT_INTERLEAVING_MISSING' }
        if ($Interleave) { Assert-HeavySamePeerInterleave $Data }
        $trace = @()
        foreach ($row in $readbackRows) {
            $readback = & (Join-Path $Repo 'target/release/x4-bridge.exe') --readback --data-dir $Data --section-key $row[0] --section-revision $row[1]
            if ($LASTEXITCODE -ne 0) { throw 'HEAVY_CONFIGURED_INDEPENDENT_READBACK_FAILED' }
            $value = ($readback -join '') | ConvertFrom-Json
            $count = if ($Throughput) { 129 } else { 1 }
            if ($value.section_revision -ne $row[1] -or @($value.records).Count -ne $count) { throw 'HEAVY_CONFIGURED_READBACK_MISMATCH' }
            $content = $value.records[0].content
            $trace += [pscustomobject]@{ key = $row[0]; value = $value }
            $snapshotRevision = if ($row[0] -like 'ship_cargo:*') { [int]$row[1] - 1 } elseif ($row[0] -like 'ship_crew:*') { [int]$row[1] - 2 } elseif ($row[0] -like 'ship_loadout:*') { [int]$row[1] - 3 } else { [int]$row[1] }
            if ($row[0] -like 'ship_crew:*' -and $content -notmatch "capacity_people=$($snapshotRevision + 12)(`n|$)") { throw 'HEAVY_CONFIGURED_CREW_VALUE_MISMATCH' }
            if ($row[0] -like 'ship_loadout:*' -and ($content -notmatch "missile=missile_ware\|missile_macro\|-$snapshotRevision(`n|$)" -or $content -notmatch 'units_selector=false')) { throw 'HEAVY_CONFIGURED_LOADOUT_VALUE_MISMATCH' }
            if ($Throughput -and $row[0] -like 'ship_cargo:*' -and @($content -split "`n" | Where-Object { $_ -like 'ware=*' }).Count -ne 80) { throw 'THROUGHPUT_NESTED_CARGO_LOSS' }
            if ($Throughput -and $row[0] -like 'ship_crew:*' -and @($content -split "`n" | Where-Object { $_ -like 'role=*' }).Count -ne 80) { throw 'THROUGHPUT_NESTED_CREW_LOSS' }
            if ($Throughput -and $row[0] -like 'ship_loadout:*' -and @($content -split "`n" | Where-Object { $_ -like 'software=*' }).Count -ne 80) { throw 'THROUGHPUT_NESTED_LOADOUT_LOSS' }
            if ($Throughput) { Write-Output "READBACK section=$($row[0]) revision=$($row[1]) records=$count content_bytes=$([Text.Encoding]::UTF8.GetByteCount(($value.records.content -join "`n")))" }
        }
        if ($Throughput) {
            $expectedTrace = if ($Interleave) { 2..20 } else { @(2..9) + @(11..14) }
            Assert-HeavyCapturedTrace $trace $expectedTrace
        }
        Write-Output 'PASS heavy-ship-recovery actual_lua_dll_pipe_production=true earlier_current_reopen=true runtime_acceptance=pending'
    } finally { Stop-Owned $producer; Stop-Owned $bridge }
    if (-not $Throughput) { foreach ($family in @('core', 'cargo', 'crew', 'loadout')) { Invoke-HeavyInterruptedRestart $Run $HostExecutable $Limits $Repo $family } }
}

function Invoke-HeavyInterruptedRestart([string]$Run, [string]$HostExecutable, [string]$Limits, [string]$Repo, [string]$Family) {
    $data = Join-Path $Run "failure-$Family"
    [IO.Directory]::CreateDirectory($data) | Out-Null
    $result = Join-Path $Run "failure-$Family.txt"
    $script = Join-Path $Repo 'extensions/live_galaxy/tests/carrier_b_heavy_configured.lua'
    $bridge = $null; $producer = $null
    try {
        $bridge = Start-Owned (Join-Path $Repo 'target/release/x4-bridge.exe') @('--data-dir', $data, '--limits-file', $Limits, '--ship-faction', 'argon') $Run
        foreach ($mode in @('baseline', "fail_$Family", 'restart')) {
            $producer = Start-Owned $HostExecutable @($Run, $script, $result, '', $mode) $Run $true
            $deadline = [DateTime]::UtcNow.AddMilliseconds((Get-Content -LiteralPath $Limits -Raw | ConvertFrom-Json).admission_window_millis)
            while (-not $producer.HasExited -and [DateTime]::UtcNow -lt $deadline) { Start-Sleep -Milliseconds 10; $producer.Refresh() }
            if (-not $producer.HasExited) { throw "HEAVY_INTERRUPTED_WATCHDOG:$Family`:$mode" }
            $producer.WaitForExit()
            if ($mode -like 'fail_*') {
                $failure = $producer.StandardError.ReadToEnd()
                if ($producer.ExitCode -eq 0 -or $failure -notmatch 'source_failure') { throw "HEAVY_SOURCE_FAILURE_MISSING:$Family`:$failure" }
            } elseif ($producer.ExitCode -ne 0) { throw "HEAVY_INTERRUPTED_RECOVERY_FAILED:$Family`:$mode`:$($producer.StandardError.ReadToEnd())" }
        }
        $events = @(Get-Content -LiteralPath (Join-Path $data 'operational-history.jsonl') | ForEach-Object { $_ | ConvertFrom-Json })
        $commits = @($events | Where-Object state -CEQ committed)
        $distinct = @($commits | ForEach-Object { "$($_.section):$($_.revision)" } | Sort-Object -Unique)
        if ($commits.Count -ne $distinct.Count) { throw 'HEAVY_DUPLICATE_PUBLICATION' }
        foreach ($key in @('ship_core:argon', 'ship_cargo:argon:g0', 'ship_crew:argon:g0', 'ship_loadout:argon:g0')) {
            $earlier = $commits | Where-Object section -CEQ $key | Select-Object -First 1
            $current = $commits | Where-Object section -CEQ $key | Select-Object -Last 1
            foreach ($entry in @($earlier, $current)) {
                $readback = & (Join-Path $Repo 'target/release/x4-bridge.exe') --readback --data-dir $data --section-key $key --section-revision $entry.revision
                if ($LASTEXITCODE -ne 0 -or (($readback -join '') | ConvertFrom-Json).section_revision -ne $entry.revision) { throw 'HEAVY_INTERRUPTED_READBACK_FAILED' }
            }
        }
        Write-Output "PASS interrupted_family=$Family no_partial_completion=true distinct_producer_restart=true independent_history_current=true"
    } finally { Stop-Owned $producer; Stop-Owned $bridge }
}
