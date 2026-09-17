function Assert-HeavyCapturedTrace([object[]]$Trace, [int[]]$Expected) {
    $actual = @($Trace | ForEach-Object { [int]$_.value.section_revision })
    if (($actual -join ',') -cne ($Expected -join ',')) { throw 'THROUGHPUT_CAPTURE_COMMIT_SET_MISMATCH' }
    $core = $null
    $previousDetail = $null
    $resume = $null
    $resumedIdentity = $null
    $advanced = $false
    $replacement = $false
    foreach ($row in $Trace) {
        if ($row.key -ceq 'ship_core') {
            if ($null -ne $core) { $replacement = $true; $resume = $previousDetail }
            $core = $row.value
            continue
        }
        $content = $row.value.records[0].content
        if ($null -eq $core -or $content -notmatch "core_revision=$($core.section_revision)(`n|$)" -or $content -notmatch "member_revision=$($core.section_revision)(`n|$)") { throw 'THROUGHPUT_STALE_DEPENDENCY' }
        $ordinal = [int]($row.key -split ':g')[1]
        if ($row.value.records[0].entity_id -cne $core.records[$ordinal].entity_id) { throw 'THROUGHPUT_FAIR_MEMBER_REBINDING' }
        $identity = $row.value.records[0].entity_id
        if ($null -ne $resume) {
            $priorIdentity = $resume.value.records[0].entity_id
            $priorFamily = ($resume.key -split ':g')[0]
            $family = ($row.key -split ':g')[0]
            $surviving = @($core.records | Where-Object { $_.entity_id -ceq $priorIdentity }).Count -gt 0
            $expectedFamily = if ($surviving -and $priorFamily -ceq 'ship_cargo') { 'ship_crew' } elseif ($surviving -and $priorFamily -ceq 'ship_crew') { 'ship_loadout' } else { 'ship_cargo' }
            $expectedIdentity = $priorIdentity
            if ($expectedFamily -ceq 'ship_cargo') {
                $next = @($core.records | Where-Object { [StringComparer]::Ordinal.Compare($_.entity_id, $priorIdentity) -gt 0 })
                $expectedIdentity = if ($next.Count -gt 0) { $next[0].entity_id } else { $core.records[0].entity_id }
            }
            if ($family -cne $expectedFamily -or $identity -cne $expectedIdentity) { throw 'THROUGHPUT_CURSOR_RESUME_LOSS' }
            $resumedIdentity = $identity
            $resume = $null
        } elseif ($null -ne $resumedIdentity -and $identity -cne $resumedIdentity) { $advanced = $true }
        $previousDetail = $row
    }
    if ($replacement -and (-not $advanced -or $null -ne $resume)) { throw 'THROUGHPUT_CURSOR_LATER_MEMBER_STARVATION' }
}
