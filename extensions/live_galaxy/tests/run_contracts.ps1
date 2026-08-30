param(
    [ValidateSet('all', 'x4_discovery', 'component_discovery', 'component_discovery_guard', 'x4-admission', 'x4-package-conformance', 'x4-candidate-runner', 'x4-verification')]
    [string]$Suite = 'all',
    [string]$LuaPath
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$admissionContract = Join-Path $root 'tools/x4-verification/tests/admission_contract.ps1'
$packageConformanceContract = Join-Path $root 'tools/x4-verification/tests/package_conformance_contract.ps1'
$candidateBuildContract = Join-Path $root 'tools/x4-verification/tests/candidate_build_contract.ps1'
$evidenceRetentionContract = Join-Path $root 'tools/x4-verification/tests/evidence_retention_contract.ps1'
$candidateIsolationContract = Join-Path $root 'tools/x4-verification/tests/candidate_isolation_contract.ps1'
$ownerAuthorityContract = Join-Path $root 'tools/x4-verification/tests/owner_authority_contract.ps1'
$ownerAuthorityAdversarial = Join-Path $root 'tools/x4-verification/tests/owner_authority_adversarial.ps1'
$evidenceChainAdversarial = Join-Path $root 'tools/x4-verification/tests/evidence_chain_adversarial.ps1'

if ($Suite -in @('all', 'x4-admission', 'x4-verification')) {
    foreach ($admissionCase in @('dossier', 'negative-fixtures', 'admission', 'evidence-chain')) {
        $admissionOutput = @(& pwsh -NoProfile -File $admissionContract -Case $admissionCase)
        if ($LASTEXITCODE -ne 0) { throw "X4 admission contract failed: $admissionCase" }
        $admissionOutput | Write-Output
        if ($admissionCase -eq 'admission' -and
            $admissionOutput -notcontains 'PASS: owner override admission contract') {
            throw 'X4 owner override admission marker was not reached.'
        }
    }
    if ($Suite -eq 'x4-admission') {
        exit 0
    }
}

if ($Suite -in @('all', 'x4-verification')) {
    & pwsh -NoProfile -File $ownerAuthorityContract -Case root-delegation
    if ($LASTEXITCODE -ne 0) { throw 'X4 owner authority root-delegation contract failed.' }
    & pwsh -NoProfile -File $ownerAuthorityAdversarial
    if ($LASTEXITCODE -ne 0) { throw 'X4 owner authority adversarial contract failed.' }
    & pwsh -NoProfile -File $candidateBuildContract -Case all
    if ($LASTEXITCODE -ne 0) { throw 'X4 candidate-build aggregate contract failed.' }
    foreach ($retentionCase in @('retention', 'retention-platform', 'handback', 'retention-admission')) {
        & pwsh -NoProfile -File $evidenceRetentionContract -Case $retentionCase
        if ($LASTEXITCODE -ne 0) { throw "X4 evidence-retention contract failed: $retentionCase" }
    }
    & pwsh -NoProfile -File $evidenceChainAdversarial
    if ($LASTEXITCODE -ne 0) { throw 'X4 held-out evidence-chain adversarial contract failed.' }
}

if ($Suite -in @('all', 'x4-candidate-runner', 'x4-verification')) {
    & pwsh -NoProfile -File $candidateIsolationContract -Case all
    if ($LASTEXITCODE -ne 0) { throw 'X4 candidate isolation contract failed.' }
}

if ($Suite -in @('all', 'x4-package-conformance', 'x4-verification')) {
    & pwsh -NoProfile -File $packageConformanceContract -Case packaged-path
    if ($LASTEXITCODE -ne 0) { throw 'X4 package conformance contract failed.' }
    if ($Suite -eq 'x4-package-conformance') {
        exit 0
    }
}

if ($Suite -eq 'component_discovery_guard') {
    & powershell -NoProfile -File (Join-Path $PSScriptRoot 'component_discovery_package_guard.ps1') -SelfTest
    if ($LASTEXITCODE -ne 0) { throw 'Component discovery package guard failed.' }
    exit 0
}

if ($Suite -eq 'component_discovery') {
    & powershell -NoProfile -File (Join-Path $PSScriptRoot 'component_discovery_binding_contract.ps1')
    if ($LASTEXITCODE -ne 0) { throw 'Component discovery binding contract failed.' }
    $componentDiscoveryProductionPaths = @(
        'extensions/live_galaxy/lua/live_galaxy_component_discovery.lua',
        'extensions/live_galaxy/lua/live_galaxy_x4_discovery.lua'
    )
    & powershell -NoProfile -File (Join-Path $PSScriptRoot 'component_discovery_package_guard.ps1') -SelfTest
    if ($LASTEXITCODE -ne 0) { throw 'Component discovery package guard self-test failed.' }
    foreach ($productionPath in $componentDiscoveryProductionPaths) {
        & powershell -NoProfile -File (Join-Path $PSScriptRoot 'component_discovery_package_guard.ps1') -ProductionPath $productionPath
        if ($LASTEXITCODE -ne 0) { throw "Component discovery package guard failed: $productionPath" }
    }
}

$lockPath = Join-Path $root 'tools/lua-runner.lock.json'
if (-not (Test-Path -LiteralPath $lockPath -PathType Leaf)) {
    throw "Lua runner lock is missing: $lockPath"
}
$lock = Get-Content -LiteralPath $lockPath -Raw | ConvertFrom-Json
if ([string]::IsNullOrWhiteSpace($lock.executableVersion) -or
    [string]::IsNullOrWhiteSpace($lock.materialization.executableRelativePath)) {
    throw 'Lua runner lock has required empty fields.'
}

function Test-LockedLua([string]$Candidate, [string]$Source) {
    if ([string]::IsNullOrWhiteSpace($Candidate) -or
        -not (Test-Path -LiteralPath $Candidate -PathType Leaf)) {
        return $null
    }

    $previousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        $reported = (& $Candidate -v 2>&1 | ForEach-Object {
            if ($_ -is [System.Management.Automation.ErrorRecord]) {
                $_.Exception.Message
            }
            else {
                $_.ToString()
            }
        } | Out-String).Trim()
    }
    finally {
        $ErrorActionPreference = $previousErrorActionPreference
    }
    if ($LASTEXITCODE -ne 0 -or
        $reported -notmatch "(?m)^$([regex]::Escape($lock.executableVersion))(?:\s|$)") {
        throw "Lua runner from $Source does not match lock '$($lock.executableVersion)': $reported"
    }
    return (Resolve-Path -LiteralPath $Candidate).Path
}

$lua = Test-LockedLua $LuaPath '-LuaPath'
if ($null -eq $lua) {
    $lua = Test-LockedLua $env:LIVE_GALAXY_LUA 'LIVE_GALAXY_LUA'
}
if ($null -eq $lua) {
    $lua = Test-LockedLua (Join-Path $root $lock.materialization.executableRelativePath) 'project-local lock path'
}
if ($null -eq $lua) {
    $pathLua = Get-Command lua -ErrorAction SilentlyContinue
    if ($null -ne $pathLua) {
        $lua = Test-LockedLua $pathLua.Source 'PATH'
    }
}
if ($null -eq $lua) {
    throw "Lua runner unavailable. Set -LuaPath or LIVE_GALAXY_LUA, or run tools/provision-lua.ps1; every candidate must match '$($lock.executableVersion)'."
}

$tests = if ($Suite -eq 'x4_discovery') {
    @('x4_discovery_contract.lua')
} elseif ($Suite -eq 'component_discovery') {
    @('component_discovery_contract.lua')
} elseif ($Suite -in @('x4-candidate-runner', 'x4-verification')) {
    @('x4_candidate_runner_contract.lua', 'x4_candidate_runner_adversarial.lua')
} else {
    @(
        Get-ChildItem $PSScriptRoot -Filter '*_contract.lua' | ForEach-Object Name
        'x4_candidate_runner_adversarial.lua'
    )
}

$candidateMarkers = @()
foreach ($test in $tests) {
    $path = Join-Path $PSScriptRoot $test
    $modulePath = Join-Path $root 'extensions\?.lua'
    $moduleInitPath = Join-Path $root 'extensions\?\init.lua'
    $extensionLuaPath = Join-Path $root 'extensions\live_galaxy\lua\?.lua'
    $caseOutput = @(& $lua -e "package.path = [[${modulePath};${moduleInitPath};${extensionLuaPath};]] .. package.path local cases = dofile([[${path}]]) for name, case in pairs(cases) do case() print('PASS x4-candidate-runner:' .. name) end")
    if ($LASTEXITCODE -ne 0) { throw "Lua contract failed: $test" }
    if ($caseOutput.Count -eq 0) { throw "Lua contract produced no behavior markers: $test" }
    $caseOutput | ForEach-Object { Write-Output $_ }
    if ($test -in @('x4_candidate_runner_contract.lua', 'x4_candidate_runner_adversarial.lua')) {
        $candidateMarkers += $caseOutput
    }
}

if ($Suite -in @('all', 'x4-candidate-runner', 'x4-verification')) {
    $requiredCandidateMarkers = @(
        'fixed_sha256_vectors_reject_same_length_tampering',
        'emits_one_candidate_as_three_digest_bound_jsonl_stages',
        'isolates_exceptions_malformed_results_and_work_unit_exhaustion',
        'lua_instruction_watchdog_preempts_cooperative_execution_and_continues',
        'never_passes_a_valid_but_unexpected_effect',
        'records_protected_contract_and_effect_failure_reasons_then_continues',
        'rejects_missing_identity_bounds_and_digest_failures_with_exact_codes',
        'independent_contract_rejects_collapsed_verdicts_and_noncanonical_order',
        'rejects_candidate_owned_authority_and_unattested_native_dispatch',
        'candidate_authority_fields_never_execute',
        'candidate_metatable_authority_never_executes',
        'direct_native_and_dynamic_authority_fail_closed',
        'runner_source_has_no_direct_native_binding'
    )
    foreach ($marker in $requiredCandidateMarkers) {
        if ($candidateMarkers -notcontains "PASS x4-candidate-runner:$marker") {
            throw "Required candidate behavior marker is missing: $marker"
        }
    }
}
