[CmdletBinding()]
param([switch]$SelfTest, [string]$EvidenceFile, [switch]$Preparation, [switch]$RuntimeAcceptance)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$fixture = Join-Path $PSScriptRoot 'fixtures/heavy-ship-source-contract.json'

function Test-Evidence([hashtable]$Evidence, [switch]$Preparation) {
    $errors = [System.Collections.Generic.List[string]]::new()
    $text = $Evidence | ConvertTo-Json -Depth 30 -Compress
    if ($text -match '(?i)([a-z]:[\\/]|\\\\[a-z]|/(home|tmp|Users|mnt)/|file://)') {
        $errors.Add('machine_path')
    }
    foreach ($key in @('schema_version', 'status', 'run', 'snapshots', 'rows', 'envelope')) {
        if (-not $Evidence.ContainsKey($key)) { $errors.Add('missing_source_field') }
    }
    if ($errors -contains 'missing_source_field') { return $errors.ToArray() }
    $expectedSnapshots = @{
        game = @('x4-9.00-steam-23660954', 'blake3:8d30413b740369e8572d6a7647366be338421698596eee84d62fec0415685c79', 'game')
        declaration = @('x4-9.00-steam-23660954-faction-ship-observation-v1', 'blake3:5cc981dc8a15914c993062aaefc37ebfc45401339d93526dd3df295ba51d996d', 'game')
        detail = @('x4-9.00-steam-23660954-ship-detail-source-v1', 'blake3:4f04a991bd595ff9493dc025a2f636fa64a6c59ed213c0dc1b19d3598b97b518', 'game')
        runtime = @('x4-probe-faction-ship-runtime-686318dd-v1', 'blake3:01ea3b0d39afee3541a1c7e87ce2c1c8fc011ce688584966952c0d20e7a90c82', 'documentation')
    }
    foreach ($name in $expectedSnapshots.Keys) {
        $s = $Evidence.snapshots[$name]
        if ($null -eq $s -or $s.id -ne $expectedSnapshots[$name][0] -or
            $s.root -ne $expectedSnapshots[$name][1] -or $s.kind -ne $expectedSnapshots[$name][2]) {
            $errors.Add('snapshot_boundary')
        }
    }
    if ($Evidence.source_snapshot.id -ne $expectedSnapshots.declaration[0] -or
        $Evidence.runtime_snapshot.id -ne $expectedSnapshots.runtime[0]) { $errors.Add('snapshot_boundary') }
    if ($Evidence.status -notin @('pending', 'observed')) { $errors.Add('untraced_observation') }
    $runTraced = $false
    if ($Evidence.run -is [hashtable]) {
        $r = $Evidence.run
        $runTraced = -not [string]::IsNullOrWhiteSpace($r.id) -and
            $r.operator -eq 'owner' -and $r.build -eq '9.00 / Steam build 23660954' -and
            $r.envelope_approval -eq 'explicit_owner' -and @($r.evidence_ids).Count -gt 0
    }
    if ($Evidence.status -eq 'observed' -and -not $runTraced) { $errors.Add('untraced_observation') }
    $requiredRows = @(
        'faction_census', 'ship_membership', 'core_identity', 'core_owner', 'core_type_class', 'core_location',
        'cargo_wares_quantities', 'cargo_capacity', 'cargo_occupied_volume',
        'crew_role_counts', 'crew_capacity', 'crew_qualification',
        'installed_engines', 'installed_shields', 'installed_weapons', 'installed_turrets',
        'installed_software', 'installed_virtual_thruster', 'missiles', 'all_drone_unit_classes', 'faction_origin'
    )
    $ids = @($Evidence.rows | ForEach-Object { $_.id })
    if (@($ids | Select-Object -Unique).Count -ne $ids.Count -or $ids.Count -ne $requiredRows.Count) {
        $errors.Add('missing_source_field')
    }
    foreach ($id in $requiredRows) {
        if ($id -notin $ids) { $errors.Add('missing_source_field') }
    }
    $states = @('pending', 'available', 'zero', 'empty', 'absent', 'not_applicable',
        'stale', 'inaccessible', 'unknown', 'unsupported')
    foreach ($row in $Evidence.rows) {
        foreach ($key in @('id', 'query', 'source_snapshot', 'runtime_snapshot', 'claim_kind', 'getter',
            'input_shape', 'output_shape', 'units', 'numeric_representation', 'applicability_evidence',
            'observation_outcome', 'capture_interval', 'source_strength', 'faction_origin_evidence',
            'stop_reason', 'provenance_ids', 'evidence_ids', 'value')) {
            if (-not $row.ContainsKey($key)) { $errors.Add('missing_source_field') }
        }
        if ($errors -contains 'missing_source_field') { continue }
        if (@($row.provenance_ids).Count -eq 0 -or
            @($row.provenance_ids | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count) {
            $errors.Add('missing_provenance')
        }
        if ($row.source_snapshot -notin @($expectedSnapshots.Values | ForEach-Object { $_[0] }) -or
            $row.runtime_snapshot -ne $expectedSnapshots.runtime[0]) { $errors.Add('snapshot_boundary') }
        if ($row.observation_outcome -notin $states) { $errors.Add('merged_outcome') }
        if ($row.observation_outcome -eq 'pending') {
            if ($null -ne $row.value -or @($row.evidence_ids).Count -or
                $null -ne $row.capture_interval.core -or $null -ne $row.capture_interval.detail) {
                $errors.Add('untraced_observation')
            }
        } elseif ($Evidence.status -ne 'observed' -or -not $runTraced -or @($row.evidence_ids).Count -eq 0) {
            $errors.Add('untraced_observation')
        }
        if ($row.observation_outcome -eq 'zero' -and
            ($row.value -is [array] -or $null -eq $row.value -or $row.value -is [string] -or $row.value -cne 0)) {
            $errors.Add('merged_outcome')
        }
        if ($row.observation_outcome -eq 'empty' -and
            ($row.value -isnot [array] -or $row.value.Count -ne 0)) { $errors.Add('merged_outcome') }
        if ($row.observation_outcome -in @('absent', 'not_applicable', 'unknown', 'inaccessible', 'unsupported') -and
            $null -ne $row.value) { $errors.Add('merged_outcome') }
        if ($row.observation_outcome -eq 'not_applicable' -and
            $row.applicability_evidence -in @('', 'pending', 'unknown')) { $errors.Add('merged_outcome') }
        if (-not $Preparation -and ($row.observation_outcome -in @('pending', 'unknown', 'inaccessible', 'unsupported', 'stale') -or
            $row.units -eq 'unknown' -or $row.input_shape -match 'source-blocked' -or
            $row.output_shape -match 'source-blocked' -or $row.applicability_evidence -eq 'pending')) {
            $errors.Add('pending_implementation')
        }
    }
    if (-not $Preparation -and ($Evidence.status -ne 'observed' -or $Evidence.envelope.status -ne 'approved')) {
        $errors.Add('pending_implementation')
    }
    return @($errors.ToArray() | Select-Object -Unique)
}

function Invoke-SelfTests {
    $cases = @(
        @{ Name = 'absent_provenance_is_rejected'; Reason = 'missing_provenance'; Change = { param($e) $e.rows[0].provenance_ids = @() } },
        @{ Name = 'fabricated_success_is_rejected'; Reason = 'untraced_observation'; Change = { param($e) $e.status = 'observed' } },
        @{ Name = 'missing_source_field_is_rejected'; Reason = 'missing_source_field'; Change = { param($e) $e.rows[0].Remove('getter') } },
        @{ Name = 'machine_path_is_rejected'; Reason = 'machine_path'; Change = { param($e) $e.rows[0].units = 'C:\Users\private\capture' } },
        @{ Name = 'merged_outcomes_are_rejected'; Reason = 'merged_outcome'; Change = { param($e) $e.rows[0].observation_outcome = 'zero_empty_not_applicable_unknown' } },
        @{ Name = 'pending_is_not_implementation_ready'; Reason = 'pending_implementation'; Change = { param($e) } }
        @{ Name = 'zero_cannot_mean_empty'; Reason = 'merged_outcome'; Change = { param($e) $e.rows[0].observation_outcome = 'zero'; $e.rows[0].value = @() } }
        @{ Name = 'empty_cannot_mean_zero'; Reason = 'merged_outcome'; Change = { param($e) $e.rows[0].observation_outcome = 'empty'; $e.rows[0].value = 0 } }
        @{ Name = 'na_requires_applicability_evidence'; Reason = 'merged_outcome'; Change = { param($e) $e.rows[0].observation_outcome = 'not_applicable' } }
        @{ Name = 'unknown_cannot_mean_zero'; Reason = 'merged_outcome'; Change = { param($e) $e.rows[0].observation_outcome = 'unknown'; $e.rows[0].value = 0 } }
        @{ Name = 'missing_required_row_is_rejected'; Reason = 'missing_source_field'; Change = { param($e) $e.rows = $e.rows[1..20] } }
        @{ Name = 'probe_snapshot_cannot_be_engine_source'; Reason = 'snapshot_boundary'; Change = { param($e) $e.snapshots.runtime.kind = 'game' } }
    )
    $failed = 0
    $index = 0
    $index++
    $preparationOutput = & pwsh -NoProfile -File $PSCommandPath -EvidenceFile $fixture -Preparation
    if ($LASTEXITCODE -eq 0) { Write-Output "ok $index - preparation_allows_pending_runtime" }
    else {
        $failed++
        Write-Output "not ok $index - preparation_allows_pending_runtime"
        Write-Output "# assertion failed: preparation must accept source-valid pending runtime; actual $preparationOutput"
    }
    $index++
    $runtimeOutput = & pwsh -NoProfile -File $PSCommandPath -EvidenceFile $fixture -RuntimeAcceptance
    if ($LASTEXITCODE -ne 0 -and $runtimeOutput -match 'pending_implementation') {
        Write-Output "ok $index - runtime_acceptance_rejects_pending_runtime"
    } else {
        $failed++
        Write-Output "not ok $index - runtime_acceptance_rejects_pending_runtime"
        Write-Output "# assertion failed: pending runtime cannot close a runtime gate; actual $runtimeOutput"
    }
    $valid = Get-Content -LiteralPath $fixture -Raw | ConvertFrom-Json -AsHashtable
    $index++
    $validErrors = @(Test-Evidence $valid -Preparation)
    if ($validErrors.Count -eq 0) { Write-Output "ok $index - sanitized_pending_shape_is_accepted" }
    else { $failed++; Write-Output "not ok $index - sanitized_pending_shape_is_accepted"; Write-Output "# assertion failed: $($validErrors -join ',')" }
    foreach ($case in $cases) {
        $index++
        $e = Get-Content -LiteralPath $fixture -Raw | ConvertFrom-Json -AsHashtable
        & $case.Change $e | Out-Null
        $errors = @(Test-Evidence $e -Preparation:($case.Name -ne 'pending_is_not_implementation_ready'))
        if ($errors -contains $case.Reason) { Write-Output "ok $index - $($case.Name)" }
        else {
            $failed++
            Write-Output "not ok $index - $($case.Name)"
            Write-Output "# assertion failed: expected $($case.Reason); actual [$($errors -join ',')]"
        }
    }
    $index++
    $buffer = [ulong[]]::new(129)
    $buffer[128] = [ulong]::MaxValue
    if ([System.Buffer]::ByteLength($buffer) -eq 1032 -and $buffer[128] -eq [ulong]::MaxValue) {
        Write-Output "ok $index - local_129_uint64_allocation_is_1032_bytes"
    } else { $failed++; Write-Output "not ok $index - local_129_uint64_allocation_is_1032_bytes"; Write-Output '# assertion failed: local allocation arithmetic' }
    Write-Output "1..$index"
    Write-Output "# tests $index"
    Write-Output "# pass $($index - $failed)"
    Write-Output "# fail $failed"
    if ($failed) { exit 1 }
}

if ($SelfTest) { Invoke-SelfTests; exit 0 }
if ($Preparation -and $RuntimeAcceptance) { throw 'EVIDENCE_MODE_CONFLICT' }
if (-not $EvidenceFile) { throw 'Specify -SelfTest or -EvidenceFile with -Preparation or -RuntimeAcceptance.' }
$evidence = Get-Content -LiteralPath $EvidenceFile -Raw | ConvertFrom-Json -AsHashtable
$errors = @(Test-Evidence $evidence -Preparation:$Preparation)
if ($errors.Count) { Write-Output "EVIDENCE_BLOCKED: $($errors -join ',')"; exit 1 }
Write-Output 'EVIDENCE_SHAPE_OK: artifact consistency only; no runtime authentication or approval'
