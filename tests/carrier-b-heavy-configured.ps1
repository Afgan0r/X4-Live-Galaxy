function Invoke-HeavyConfiguredRestart([string]$Run, [string]$HostExecutable, [string]$Data, [string]$Limits, [string]$Repo, [switch]$Throughput, [switch]$Interleave) {
    if ($Interleave -and -not $Throughput) { throw 'INTERLEAVE_REQUIRES_LOCAL_THROUGHPUT' }
    $bridge = $null; $producer = $null
    $result = Join-Path $Run 'heavy-result.txt'
    $script = Join-Path $Repo 'extensions/live_galaxy/tests/carrier_b_heavy_configured.lua'
    $readbackRows = @()
    try {
        $bridge = Start-Owned (Join-Path $Repo 'target/release/x4-bridge.exe') @('--data-dir', $Data, '--limits-file', $Limits, '--ship-faction', 'argon') $Run
        $modes = if ($Interleave) { @('first') } else { @('first', 'restart') }
        foreach ($mode in $modes) {
            $marker = if ($Interleave) { 'throughput-interleave' } elseif ($Throughput) { 'throughput' } else { '' }
            $producer = Start-Owned $HostExecutable @($Run, $script, $result, $marker, $mode) $Run $true
            $deadline = [DateTime]::UtcNow.AddMilliseconds((Get-Content -LiteralPath $Limits -Raw | ConvertFrom-Json).admission_window_millis)
            while (-not $producer.HasExited -and [DateTime]::UtcNow -lt $deadline) { Start-Sleep -Milliseconds 10; $producer.Refresh() }
            if (-not $producer.HasExited) { throw 'HEAVY_CONFIGURED_HOST_WATCHDOG' }
            $producer.WaitForExit()
            if ($producer.ExitCode -ne 0) { throw "HEAVY_CONFIGURED_HOST_FAILED:$($producer.StandardError.ReadToEnd())" }
            $expected = if ($Interleave) { (1..19) -join ',' } elseif ($mode -ceq 'first') { '1,2,3,4,5,6,7,8' } else { '9,10,11,12' }
            if (-not (Select-String -LiteralPath $result -SimpleMatch "revisions=$expected" -Quiet)) { throw 'HEAVY_CONFIGURED_REVISION_MISMATCH' }
            if ($Throughput) {
                Get-Content -LiteralPath $result
                foreach ($line in Get-Content -LiteralPath $result) {
                    if ($line -match '^capture section=(\S+) revision=(\d+) ') { $readbackRows += ,@($Matches[1], [int]$Matches[2]) }
                }
            }
        }
        if (-not $Throughput) { $readbackRows = @(@('ship_core', 1), @('ship_core', 9), @('ship_cargo:g0', 2), @('ship_cargo:g0', 10), @('ship_crew:g0', 3), @('ship_crew:g0', 11), @('ship_loadout:g0', 4), @('ship_loadout:g0', 12)) }
        if ($Interleave -and @($readbackRows | Where-Object { $_[0] -ceq 'ship_core' }).Count -lt 2) { throw 'THROUGHPUT_PARENT_INTERLEAVING_MISSING' }
        $currentCore = $null
        foreach ($row in $readbackRows) {
            $readback = & (Join-Path $Repo 'target/release/x4-bridge.exe') --readback --data-dir $Data --section-key $row[0] --section-revision $row[1]
            if ($LASTEXITCODE -ne 0) { throw 'HEAVY_CONFIGURED_INDEPENDENT_READBACK_FAILED' }
            $value = ($readback -join '') | ConvertFrom-Json
            $count = if ($Throughput -and $row[0] -ceq 'ship_core') { 129 } else { 1 }
            if ($value.section_revision -ne $row[1] -or @($value.records).Count -ne $count) { throw 'HEAVY_CONFIGURED_READBACK_MISMATCH' }
            $content = $value.records[0].content
            if ($Throughput -and $row[0] -ceq 'ship_core') { $currentCore = $value }
            elseif ($Throughput) {
                if ($content -notmatch "core_revision=$($currentCore.section_revision)(`n|$)" -or $content -notmatch "member_revision=$($currentCore.section_revision)(`n|$)") { throw 'THROUGHPUT_STALE_DEPENDENCY' }
                $ordinal = [int]($row[0] -split ':g')[1]
                if ($value.records[0].entity_id -cne $currentCore.records[$ordinal].entity_id) { throw 'THROUGHPUT_FAIR_MEMBER_REBINDING' }
            }
            if ($row[0] -like 'ship_crew:*' -and $content -notmatch "capacity_people=$([int]$row[1] + 12)(`n|$)") { throw 'HEAVY_CONFIGURED_CREW_VALUE_MISMATCH' }
            if ($row[0] -like 'ship_loadout:*' -and ($content -notmatch "missile=missile_ware\|missile_macro\|-$($row[1])(`n|$)" -or $content -notmatch 'units_selector=false')) { throw 'HEAVY_CONFIGURED_LOADOUT_VALUE_MISMATCH' }
            if ($Throughput -and $row[0] -like 'ship_cargo:*' -and @($content -split "`n" | Where-Object { $_ -like 'ware=*' }).Count -ne 80) { throw 'THROUGHPUT_NESTED_CARGO_LOSS' }
            if ($Throughput -and $row[0] -like 'ship_crew:*' -and @($content -split "`n" | Where-Object { $_ -like 'role=*' }).Count -ne 80) { throw 'THROUGHPUT_NESTED_CREW_LOSS' }
            if ($Throughput -and $row[0] -like 'ship_loadout:*' -and @($content -split "`n" | Where-Object { $_ -like 'software=*' }).Count -ne 80) { throw 'THROUGHPUT_NESTED_LOADOUT_LOSS' }
            if ($Throughput) { Write-Output "READBACK section=$($row[0]) revision=$($row[1]) records=$count content_bytes=$([Text.Encoding]::UTF8.GetByteCount(($value.records.content -join "`n")))" }
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
        foreach ($key in @('ship_core', 'ship_cargo:g0', 'ship_crew:g0', 'ship_loadout:g0')) {
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
