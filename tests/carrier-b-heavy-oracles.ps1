function Assert-HeavyCapturedTrace([object[]]$Trace, [int[]]$Expected) {
    $actual = @($Trace | ForEach-Object { [int]$_.value.section_revision })
    if (($actual -join ',') -cne ($Expected -join ',')) { throw 'THROUGHPUT_CAPTURE_COMMIT_SET_MISMATCH' }
    $core = $null
    $replacement = $false
    $expectedFamily = 'ship_cargo'
    $completedAfterReplacement = $false
    foreach ($row in $Trace) {
        if ($row.key -like 'ship_core:*') {
            if ($null -ne $core) { $replacement = $true }
            $core = $row.value
            $expectedFamily = 'ship_cargo'
            continue
        }
        $family = ($row.key -split ':')[0]
        if ($family -cne $expectedFamily) { throw 'THROUGHPUT_CURSOR_RESUME_LOSS' }
        if ($null -eq $core -or $row.value.records.Count -ne $core.records.Count) {
            throw 'THROUGHPUT_FULL_GROUP_COUNT_MISMATCH'
        }
        for ($index = 0; $index -lt $core.records.Count; $index++) {
            $detail = $row.value.records[$index]
            $content = $detail.content
            if ($content -notmatch "core_revision=$($core.section_revision)(`n|$)" -or
                $content -notmatch "member_revision=$($core.section_revision)(`n|$)") {
                throw 'THROUGHPUT_STALE_DEPENDENCY'
            }
            if ($detail.entity_id -cne $core.records[$index].entity_id) {
                throw 'THROUGHPUT_FULL_GROUP_MEMBERSHIP_MISMATCH'
            }
        }
        $expectedFamily = switch ($family) {
            'ship_cargo' { 'ship_crew' }
            'ship_crew' { 'ship_loadout' }
            'ship_loadout' { if ($replacement) { $completedAfterReplacement = $true }; 'ship_cargo' }
            default { throw 'THROUGHPUT_UNKNOWN_FAMILY' }
        }
    }
    if ($replacement -and -not $completedAfterReplacement) { throw 'THROUGHPUT_REPLACEMENT_STARVATION' }
}
