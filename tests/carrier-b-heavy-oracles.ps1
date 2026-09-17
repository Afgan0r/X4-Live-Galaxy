function Assert-HeavyCapturedTrace([object[]]$Trace, [int[]]$Expected) {
    $core = $null
    foreach ($row in $Trace) {
        if ($row.key -ceq 'ship_core') { $core = $row.value; continue }
        $content = $row.value.records[0].content
        if ($null -eq $core -or $content -notmatch "core_revision=$($core.section_revision)(`n|$)" -or $content -notmatch "member_revision=$($core.section_revision)(`n|$)") { throw 'THROUGHPUT_STALE_DEPENDENCY' }
        $ordinal = [int]($row.key -split ':g')[1]
        if ($row.value.records[0].entity_id -cne $core.records[$ordinal].entity_id) { throw 'THROUGHPUT_FAIR_MEMBER_REBINDING' }
    }
}
