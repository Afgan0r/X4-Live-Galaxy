[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string]$EvidencePath,
    [ValidateSet('skeleton', 'full')]
    [string]$Stage = 'full'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Fail([string]$Message) {
    throw "Evidence validation failed: $Message"
}

function Require-Text($Value, [string]$Name) {
    if ([string]::IsNullOrWhiteSpace([string]$Value)) {
        Fail "$Name must be non-empty."
    }
}

function Require-Array($Value, [string]$Name) {
    if ($null -eq $Value -or @($Value).Count -eq 0) {
        Fail "$Name must be a non-empty array."
    }
}

function Require-ExactSet($Values, [string[]]$Expected, [string]$Name) {
    $actual = @($Values | Sort-Object -Unique)
    $wanted = @($Expected | Sort-Object -Unique)
    if (($actual -join '|') -ne ($wanted -join '|')) {
        Fail "$Name must be exactly: $($Expected -join ', ')."
    }
}

function Assert-UniqueIds($Items, [string]$Kind) {
    $ids = @($Items | ForEach-Object { [string]$_.id })
    if (@($ids | Sort-Object -Unique).Count -ne $ids.Count) {
        Fail "$Kind IDs must be unique."
    }
}

function Get-ClaimRegister([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Fail "Evidence file does not exist: $Path"
    }
    $content = Get-Content -LiteralPath $Path -Raw -Encoding utf8
    $matches = [regex]::Matches(
        $content,
        '(?ms)^```json hostile-claim-register\s*\r?\n(?<json>.*?)^```\s*$'
    )
    if ($matches.Count -ne 1) {
        Fail 'Exactly one json hostile-claim-register fenced payload is required.'
    }
    try {
        return $matches[0].Groups['json'].Value | ConvertFrom-Json
    }
    catch {
        Fail "Claim register JSON is malformed: $($_.Exception.Message)"
    }
}

function Get-SourceAllowlist {
    return @{
        'x4-version-dat' = @{ kind = 'installed_x4_file'; path = 'version.dat'; boundary = 'installed X4 9.00'; conclusions = @('installed_version') }
        'x4-jobs' = @{ kind = 'installed_x4_file'; path = '08.cat::libraries/jobs.xml'; boundary = 'installed X4 9.00 catalog'; conclusions = @('xen_job_configuration', 'xen_patrol_configuration') }
        'x4-khaak-activity' = @{ kind = 'installed_x4_file'; path = '08.cat::md/khaak_activity.xml'; boundary = 'installed X4 9.00 catalog'; conclusions = @('khk_activity_configuration', 'khk_spawn_configuration') }
        'khaakfinder-precedent' = @{ kind = 'installed_extension_precedent'; path = 'extensions/z_ram_khaakfinder'; boundary = 'installed extension v101'; conclusions = @('visibility_precedent') }
        'project-contract' = @{ kind = 'repository_contract'; path = 'AGENTS.md|.planning/PROJECT.md|.planning/REQUIREMENTS.md|.planning/ROADMAP.md|02-CONTEXT.md'; boundary = 'repository planning'; conclusions = @('observation_only_scope', 'runtime_contract_unknown', 'non_gating_deferral') }
    }
}

function Test-Register($Register, [string]$ValidationStage) {
    foreach ($field in @('schema_version', 'source_boundary', 'scope', 'coverage', 'sources', 'claims')) {
        if ($null -eq $Register.$field) { Fail "Top-level field $field is required." }
    }
    Require-Text $Register.schema_version 'schema_version'
    Require-Text $Register.source_boundary 'source_boundary'
    $scopeNames = @(
        'xen_primary_pressure', 'khk_observed_when_present', 'autonomous_hostile_minds',
        'government_institutions', 'hostile_motives', 'hostile_diplomacy',
        'hostile_architecture_selected', 'hostile_write_primitives',
        'hostile_control_channels', 'critical_path_dependency', 'phase8_inventory_only'
    )
    foreach ($name in $scopeNames) {
        if ($Register.scope.PSObject.Properties.Name -notcontains $name -or $Register.scope.$name -isnot [bool]) {
            Fail "scope.$name must be a boolean."
        }
    }
    foreach ($name in @('autonomous_hostile_minds', 'government_institutions', 'hostile_motives', 'hostile_diplomacy', 'hostile_architecture_selected', 'hostile_write_primitives', 'hostile_control_channels', 'critical_path_dependency')) {
        if ($Register.scope.$name) { Fail "scope.$name must remain false." }
    }
    if (-not $Register.scope.xen_primary_pressure -or -not $Register.scope.khk_observed_when_present -or -not $Register.scope.phase8_inventory_only) {
        Fail 'The positive observation-only scope flags must remain true.'
    }

    Require-Array $Register.coverage.requirements 'coverage.requirements'
    Require-Array $Register.coverage.decisions 'coverage.decisions'
    Require-Array $Register.sources 'sources'
    Require-Array $Register.claims 'claims'
    Assert-UniqueIds @($Register.sources) 'Source'
    Assert-UniqueIds @($Register.claims) 'Claim'

    $allowlist = Get-SourceAllowlist
    foreach ($source in @($Register.sources)) {
        foreach ($field in @('id', 'kind', 'path', 'boundary', 'allowed_conclusions')) {
            if ($null -eq $source.$field) { Fail "Source field $field is required." }
        }
        Require-Text $source.id 'source.id'
        Require-Text $source.kind "source $($source.id) kind"
        Require-Text $source.path "source $($source.id) path"
        Require-Text $source.boundary "source $($source.id) boundary"
        Require-Array $source.allowed_conclusions "source $($source.id) allowed_conclusions"
        if (-not $allowlist.ContainsKey($source.id)) { Fail "Source ID is not allowlisted: $($source.id)" }
        $allowed = $allowlist[$source.id]
        if ($source.kind -ne $allowed.kind -or $source.path -ne $allowed.path -or $source.boundary -ne $allowed.boundary) {
            Fail "Source $($source.id) has an unallowlisted kind, path, or boundary."
        }
        foreach ($conclusion in @($source.allowed_conclusions)) {
            if ($allowed.conclusions -notcontains $conclusion) { Fail "Source $($source.id) permits an unallowlisted conclusion: $conclusion" }
        }
    }
    if (@($Register.sources).Count -ne $allowlist.Count) { Fail 'The source registry must contain every allowlisted source exactly once.' }

    $sources = @{}
    foreach ($source in @($Register.sources)) { $sources[$source.id] = $source }
    $classifications = @('documented', 'observed', 'inferred', 'unknown')
    foreach ($claim in @($Register.claims)) {
        foreach ($field in @('id', 'faction', 'area', 'classification', 'source_id', 'permitted_conclusion', 'non_gating', 'future_owner', 'evidence_needed')) {
            if ($null -eq $claim.$field) { Fail "Claim field $field is required." }
        }
        if ($classifications -notcontains $claim.classification) { Fail "Claim $($claim.id) has invalid classification." }
        if ($claim.non_gating -isnot [bool]) { Fail "Claim $($claim.id) non_gating must be boolean." }
        foreach ($field in @('id', 'faction', 'area', 'source_id', 'permitted_conclusion', 'future_owner', 'evidence_needed')) { Require-Text $claim.$field "claim.$field" }
        if (-not $sources.ContainsKey($claim.source_id)) { Fail "Claim $($claim.id) references unknown source ID $($claim.source_id)." }
        if (@($sources[$claim.source_id].allowed_conclusions) -notcontains $claim.permitted_conclusion) {
            Fail "Claim $($claim.id) conclusion is outside source scope."
        }
        if ($claim.classification -eq 'unknown' -and (-not $claim.non_gating -or [string]::IsNullOrWhiteSpace($claim.future_owner) -or [string]::IsNullOrWhiteSpace($claim.evidence_needed))) {
            Fail "Unknown claim $($claim.id) must remain non-gating with a future owner and evidence need."
        }
    }

    if ($ValidationStage -eq 'full') {
        Require-ExactSet $Register.coverage.requirements @('RES-01', 'RES-02', 'RES-03') 'coverage.requirements'
        Require-ExactSet $Register.coverage.decisions @('D-01', 'D-02', 'D-03', 'D-04', 'D-05', 'D-06', 'D-07', 'D-08') 'coverage.decisions'
        $areas = @('state', 'events', 'identity', 'visibility', 'scheduling', 'economy_or_spawning_ownership', 'control_limits')
        foreach ($faction in @('XEN', 'KHK')) {
            Require-ExactSet @($Register.claims | Where-Object { $_.faction -eq $faction } | ForEach-Object { $_.area }) $areas "$faction claim areas"
        }
    }
}

try {
    Test-Register (Get-ClaimRegister $EvidencePath) $Stage
    Write-Output "PASS: $Stage evidence validation succeeded."
}
catch {
    Write-Error $_.Exception.Message
    exit 1
}
