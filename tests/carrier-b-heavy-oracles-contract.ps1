Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'carrier-b-heavy-oracles.ps1')

function Row([string]$Key, [int]$Revision, [string]$Identity, [int]$Parent) {
    $records = if ($Key -ceq 'ship_core') {
        @([pscustomobject]@{ entity_id = 'x4:ship:1'; content = '' }, [pscustomobject]@{ entity_id = 'x4:ship:2'; content = '' })
    } else { @([pscustomobject]@{ entity_id = $Identity; content = "core_revision=$Parent`nmember_revision=$Parent" }) }
    [pscustomobject]@{ key = $Key; value = [pscustomobject]@{ section_revision = $Revision; records = $records } }
}

$valid = @((Row 'ship_core' 1 '' 0), (Row 'ship_cargo:g0' 2 'x4:ship:1' 1), (Row 'ship_core' 3 '' 0), (Row 'ship_crew:g0' 4 'x4:ship:1' 3), (Row 'ship_loadout:g0' 5 'x4:ship:1' 3), (Row 'ship_cargo:g1' 6 'x4:ship:2' 3))
Assert-HeavyCapturedTrace $valid (1..6)
$reset = @($valid)
$reset[3] = Row 'ship_cargo:g0' 4 'x4:ship:1' 3
$cases = @([pscustomobject]@{ name = 'reset_member_family'; trace = $reset }, [pscustomobject]@{ name = 'missing_capture'; trace = @($valid | Where-Object { $_.value.section_revision -ne 4 }) })
foreach ($case in $cases) {
    $rejected = $false
    try { Assert-HeavyCapturedTrace $case.trace (1..6) } catch { $rejected = $true }
    if (-not $rejected) { throw "HARNESS_ORACLE_ACCEPTED:$($case.name)" }
}
Write-Output 'PASS heavy_harness_oracles valid=true reset_cursor_rejected=true missing_capture_rejected=true'
