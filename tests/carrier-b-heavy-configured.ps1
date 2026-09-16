function Invoke-HeavyConfiguredRestart([string]$Run, [string]$HostExecutable, [string]$Data, [string]$Limits, [string]$Repo) {
    $bridge = $null; $producer = $null
    $result = Join-Path $Run 'heavy-result.txt'
    $script = Join-Path $Repo 'extensions/live_galaxy/tests/carrier_b_heavy_configured.lua'
    try {
        $bridge = Start-Owned (Join-Path $Repo 'target/release/x4-bridge.exe') @('--data-dir', $Data, '--limits-file', $Limits, '--ship-faction', 'argon') $Run
        foreach ($mode in @('first', 'restart')) {
            $producer = Start-Owned $HostExecutable @($Run, $script, $result, '', $mode) $Run $true
            $deadline = [DateTime]::UtcNow.AddMilliseconds((Get-Content -LiteralPath $Limits -Raw | ConvertFrom-Json).admission_window_millis)
            while (-not $producer.HasExited -and [DateTime]::UtcNow -lt $deadline) { Start-Sleep -Milliseconds 10; $producer.Refresh() }
            if (-not $producer.HasExited) { throw 'HEAVY_CONFIGURED_HOST_WATCHDOG' }
            $producer.WaitForExit()
            if ($producer.ExitCode -ne 0) { throw "HEAVY_CONFIGURED_HOST_FAILED:$($producer.StandardError.ReadToEnd())" }
            $expected = if ($mode -ceq 'first') { '1,2,3,4,5,6,7,8' } else { '9,10,11,12' }
            if (-not (Select-String -LiteralPath $result -SimpleMatch "revisions=$expected" -Quiet)) { throw 'HEAVY_CONFIGURED_REVISION_MISMATCH' }
        }
        foreach ($row in @(@('ship_core', 1), @('ship_core', 9), @('ship_cargo:g0', 2), @('ship_cargo:g0', 10), @('ship_crew:g0', 3), @('ship_crew:g0', 11), @('ship_loadout:g0', 4), @('ship_loadout:g0', 12))) {
            $readback = & (Join-Path $Repo 'target/release/x4-bridge.exe') --readback --data-dir $Data --section-key $row[0] --section-revision $row[1]
            if ($LASTEXITCODE -ne 0) { throw 'HEAVY_CONFIGURED_INDEPENDENT_READBACK_FAILED' }
            $value = ($readback -join '') | ConvertFrom-Json
            if ($value.section_revision -ne $row[1] -or @($value.records).Count -ne 1) { throw 'HEAVY_CONFIGURED_READBACK_MISMATCH' }
            $content = $value.records[0].content
            if ($row[0] -like 'ship_crew:*' -and $content -notmatch "capacity_people=$([int]$row[1] + 12)(`n|$)") { throw 'HEAVY_CONFIGURED_CREW_VALUE_MISMATCH' }
            if ($row[0] -like 'ship_loadout:*' -and ($content -notmatch "missile=missile_ware\|missile_macro\|-$($row[1])(`n|$)" -or $content -notmatch 'units_selector=false')) { throw 'HEAVY_CONFIGURED_LOADOUT_VALUE_MISMATCH' }
        }
        Write-Output 'PASS heavy-ship-recovery actual_lua_dll_pipe_production=true earlier_current_reopen=true runtime_acceptance=pending'
    } finally { Stop-Owned $producer; Stop-Owned $bridge }
    foreach ($family in @('core', 'cargo', 'crew', 'loadout')) { Invoke-HeavyInterruptedRestart $Run $HostExecutable $Limits $Repo $family }
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
