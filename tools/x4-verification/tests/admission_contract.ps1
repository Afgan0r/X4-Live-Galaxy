[CmdletBinding()]
param(
    [ValidateSet('dossier', 'negative-fixtures', 'admission', 'evidence-chain')]
    [string]$Case = 'dossier'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$toolRoot = Split-Path -Parent $PSScriptRoot
$root = Split-Path -Parent (Split-Path -Parent $toolRoot)
$contractsRoot = Join-Path $toolRoot 'contracts'
$admissionPath = Join-Path $toolRoot 'x4-admission.ps1'
$dossierPath = Join-Path $contractsRoot 'dossier.v1.json'
$registryPath = Join-Path $contractsRoot 'known-failures.v1.json'
$coveragePath = Join-Path $contractsRoot 'coverage.v1.json'
$fixturePath = Join-Path $toolRoot 'fixtures/negative-fixtures.v1.json'
$overrideContractPath = Join-Path $contractsRoot 'owner-override.v1.json'
$matrixPath = Join-Path $root 'tests/x4-candidates/phase-05.1-candidates.v1.json'
$ledgerPath = Join-Path $root 'tests/x4-candidates/phase-05.1-candidate-ledger.v1.json'

$requiredDimensions = @(
    'loader-registration',
    'module-resolution',
    'lua-native-binding',
    'arguments-returns',
    'native-canonical-identity',
    'lifecycle-thread',
    'cadence-reconnect',
    'failure-partial-completeness',
    'volume-performance'
)

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) {
        throw $Message
    }
}

function Copy-Json($Value) {
    return ($Value | ConvertTo-Json -Depth 32 | ConvertFrom-Json)
}

function Invoke-Admission($Dossier, $Registry, $Coverage = $null, $Fixtures = $null, $Override = $null, $Ledger = $null, $Matrix = $null) {
    $tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("live-galaxy-admission-{0}" -f [guid]::NewGuid().ToString('N'))
    [void](New-Item -ItemType Directory -Path $tempRoot)
    try {
        $tempDossier = Join-Path $tempRoot 'dossier.json'
        $tempRegistry = Join-Path $tempRoot 'registry.json'
        $Dossier | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $tempDossier -Encoding utf8NoBOM
        $inputDigestBefore = (Get-FileHash -LiteralPath $tempDossier -Algorithm SHA256).Hash.ToLowerInvariant()
        $Registry | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $tempRegistry -Encoding utf8NoBOM
        $arguments = @('-NoProfile', '-File', $admissionPath, '-DossierPath', $tempDossier, '-RegistryPath', $tempRegistry)
        if ($null -ne $Coverage) {
            $tempCoverage = Join-Path $tempRoot 'coverage.json'
            $Coverage | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $tempCoverage -Encoding utf8NoBOM
            $arguments += @('-CoveragePath', $tempCoverage)
        }
        if ($null -ne $Fixtures) {
            $tempFixtures = Join-Path $tempRoot 'fixtures.json'
            $Fixtures | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $tempFixtures -Encoding utf8NoBOM
            $arguments += @('-FixturePath', $tempFixtures)
        }
        if ($null -ne $Override) {
            $tempOverride = Join-Path $tempRoot 'override.json'
            $resolvedOverride = Copy-Json $Override
            if ($resolvedOverride.PSObject.Properties.Name -contains 'dossier_digest' -and
                $resolvedOverride.dossier_digest -eq '__DOSSIER_DIGEST__') {
                $resolvedOverride.dossier_digest = $inputDigestBefore
            }
            $resolvedOverride | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $tempOverride -Encoding utf8NoBOM
            $arguments += @('-OverridePath', $tempOverride)
        }
        if ($null -ne $Ledger) {
            $tempLedger = Join-Path $tempRoot 'ledger.json'
            $Ledger | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $tempLedger -Encoding utf8NoBOM
            $arguments += @('-SanitizedLedgerPath', $tempLedger)
        }
        if ($null -ne $Matrix) {
            $tempMatrix = Join-Path $tempRoot 'matrix.json'
            $Matrix | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $tempMatrix -Encoding utf8NoBOM
            $arguments += @('-CandidateMatrixPath', $tempMatrix)
        }
        $output = & pwsh @arguments 2>&1
        $exitCode = $LASTEXITCODE
        $inputDigestAfter = (Get-FileHash -LiteralPath $tempDossier -Algorithm SHA256).Hash.ToLowerInvariant()
        $jsonLine = @($output | ForEach-Object { $_.ToString() } | Where-Object { $_.TrimStart().StartsWith('{') }) | Select-Object -Last 1
        Assert-True ($null -ne $jsonLine) "Admission emitted no JSON result: $($output -join ' | ')"
        return [pscustomobject]@{
            ExitCode = $exitCode
            Result = $jsonLine | ConvertFrom-Json
            Output = @($output | ForEach-Object { $_.ToString() })
            InputDigestBefore = $inputDigestBefore
            InputDigestAfter = $inputDigestAfter
        }
    }
    finally {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }
}

function Assert-Rejected($Run, [string]$ReasonCode, [string]$Label) {
    Assert-True ($Run.ExitCode -ne 0) "$Label unexpectedly succeeded."
    Assert-True ($Run.Result.verdict -eq 'non-admissible') "$Label returned an unstable verdict."
    Assert-True (@($Run.Result.reason_codes) -contains $ReasonCode) "$Label did not report $ReasonCode; got $(@($Run.Result.reason_codes) -join ',') at $($Run.Result.diagnostic_id)."
    $joined = $Run.Output -join "`n"
    Assert-True ($joined -notmatch '(?i)[A-Z]:\\|/Users/|\\Users\\|private|secret') "$Label leaked a private path or value."
}

$requiredPaths = @($admissionPath, $dossierPath, $registryPath)
if ($Case -eq 'negative-fixtures') {
    $requiredPaths += @($coveragePath, $fixturePath)
}
if ($Case -eq 'admission') {
    $requiredPaths += @($coveragePath, $fixturePath, $overrideContractPath)
}
if ($Case -eq 'evidence-chain') {
    $requiredPaths += @($coveragePath, $fixturePath, $ledgerPath, $matrixPath)
}
foreach ($path in $requiredPaths) {
    Assert-True (Test-Path -LiteralPath $path -PathType Leaf) "Required admission artifact is missing: $path"
}

$baseDossier = Get-Content -LiteralPath $dossierPath -Raw -Encoding utf8 | ConvertFrom-Json
$baseRegistry = Get-Content -LiteralPath $registryPath -Raw -Encoding utf8 | ConvertFrom-Json

if ($Case -eq 'evidence-chain') {
    $coverage = Get-Content -LiteralPath $coveragePath -Raw -Encoding utf8 | ConvertFrom-Json
    $fixtures = Get-Content -LiteralPath $fixturePath -Raw -Encoding utf8 | ConvertFrom-Json
    $pendingLedger = Get-Content -LiteralPath $ledgerPath -Raw -Encoding utf8 | ConvertFrom-Json
    $matrix = Get-Content -LiteralPath $matrixPath -Raw -Encoding utf8 | ConvertFrom-Json
    $completeLedger = Copy-Json $pendingLedger
    $completeLedger.status = 'runtime-complete'
    foreach ($candidate in @($completeLedger.candidates)) {
        $candidate.status = 'retained'
        $candidate.disposition = 'production'
        $candidate.execution_verdict = 'pass'
        $candidate.contract_verdict = 'pass'
        $candidate.effect_verdict = 'pass'
        $candidate.actual_effect_id = $candidate.expected_effect_id
        $candidate.run_id = "run-$($candidate.id)"
        $candidate.identity_digests | Add-Member -NotePropertyName evidence_digest -NotePropertyValue ('a' * 64)
        $candidate.identity_digests | Add-Member -NotePropertyName locator_digest -NotePropertyValue ('b' * 64)
        $runText = "$($candidate.run_id)|$($candidate.identity_digests.evidence_digest)|$($candidate.identity_digests.build_manifest_digest)"
        $runBytes = [Text.Encoding]::UTF8.GetBytes($runText)
        $runDigest = ([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($runBytes))).ToLowerInvariant()
        $candidate.identity_digests | Add-Member -NotePropertyName run_digest -NotePropertyValue $runDigest
    }

    $accepted = Invoke-Admission $baseDossier $baseRegistry $coverage $fixtures $null $completeLedger $matrix
    Assert-True ($accepted.ExitCode -eq 0) "Complete retained evidence chain failed: $($accepted.Output -join ' | ')"
    Assert-True ($accepted.Result.verdict -eq 'admissible') 'Complete retained evidence chain was not admitted.'

    $missingEvidence = Copy-Json $completeLedger
    $missingEvidence.candidates[0].identity_digests.PSObject.Properties.Remove('evidence_digest')
    Assert-Rejected (Invoke-Admission $baseDossier $baseRegistry $coverage $fixtures $null $missingEvidence $matrix) 'EVIDENCE_CHAIN_INCOMPLETE' 'missing retained evidence digest'

    $incomplete = Copy-Json $completeLedger
    $incomplete.candidates[0].status = 'runtime-pending'
    Assert-Rejected (Invoke-Admission $baseDossier $baseRegistry $coverage $fixtures $null $incomplete $matrix) 'RETENTION_INCOMPLETE' 'incomplete retention'

    $sourceAction = Copy-Json $matrix
    $sourceAction.candidates[0].source_action_only = $true
    Assert-Rejected (Invoke-Admission $baseDossier $baseRegistry $coverage $fixtures $null $completeLedger $sourceAction) 'SOURCE_ACTION_DEFECT' 'source-action candidate'

    foreach ($axis in @('execution', 'contract', 'effect')) {
        $failed = Copy-Json $completeLedger
        $failed.candidates[0]."${axis}_verdict" = 'fail'
        Assert-Rejected (Invoke-Admission $baseDossier $baseRegistry $coverage $fixtures $null $failed $matrix) 'FAILED_RUNTIME_VERDICT' "failed $axis verdict"
    }

    $unexpected = Copy-Json $completeLedger
    $unexpected.candidates[0].actual_effect_id = 'valid-unexpected-result'
    $unexpected.candidates[0].effect_verdict = 'pass'
    Assert-Rejected (Invoke-Admission $baseDossier $baseRegistry $coverage $fixtures $null $unexpected $matrix) 'UNEXPECTED_EFFECT' 'valid unexpected effect marked pass'

    $identityMismatch = Copy-Json $completeLedger
    $identityMismatch.candidates[0].identity_digests.dossier_digest = '0' * 64
    Assert-Rejected (Invoke-Admission $baseDossier $baseRegistry $coverage $fixtures $null $identityMismatch $matrix) 'IDENTITY_CHAIN_MISMATCH' 'dossier identity mismatch'

    Write-Output 'PASS: sanitized evidence-chain admission contract'
    exit 0
}

if ($Case -eq 'admission') {
    $coverage = Get-Content -LiteralPath $coveragePath -Raw -Encoding utf8 | ConvertFrom-Json
    $fixtures = Get-Content -LiteralPath $fixturePath -Raw -Encoding utf8 | ConvertFrom-Json
    $override = Get-Content -LiteralPath $overrideContractPath -Raw -Encoding utf8 | ConvertFrom-Json
    $knownFailure = Copy-Json $baseDossier
    $knownFailure.dimensions[0] | Add-Member -NotePropertyName finding_ids -NotePropertyValue @('finding-small-loader-exception')
    $knownFailure.findings = @([pscustomobject]@{
        id = 'finding-small-loader-exception'
        failure_class_id = 'loader-mismatch'
        dimension_id = 'loader-registration'
        disposition = 'known-failure'
    })

    $blocked = Invoke-Admission $knownFailure $baseRegistry $coverage $fixtures
    Assert-Rejected $blocked 'KNOWN_FAILURE_BLOCKED' 'known failure without override'
    Assert-True ($blocked.InputDigestBefore -eq $blocked.InputDigestAfter) 'Admission mutated the source dossier.'

    $override.dossier_id = $knownFailure.dossier_id
    $override.dossier_digest = '__DOSSIER_DIGEST__'
    $override.finding_id = 'finding-small-loader-exception'
    $override.expires_at = [DateTimeOffset]::UtcNow.AddDays(30).ToString('yyyy-MM-ddTHH:mm:ssZ')
    $accepted = Invoke-Admission $knownFailure $baseRegistry $coverage $fixtures $override
    Assert-True ($accepted.ExitCode -eq 0) "Exact override failed: $($accepted.Output -join ' | ')"
    Assert-True ($accepted.Result.verdict -eq 'admissible-with-owner-override') 'Exact override returned the wrong verdict.'
    Assert-True (@($accepted.Result.reason_codes) -contains 'OWNER_OVERRIDE_APPLIED') 'Exact override returned the wrong reason.'
    Assert-True (@($accepted.Result.overridden_finding_ids) -join '|' -eq 'finding-small-loader-exception') 'Override did not report the exact finding.'
    Assert-True ($accepted.InputDigestBefore -eq $accepted.InputDigestAfter) 'Exact override mutated the source dossier.'

    foreach ($field in @('schema_version', 'override_id', 'dossier_id', 'dossier_digest', 'finding_id', 'owner_decision_id', 'decision', 'rationale', 'remaining_risk', 'expires_at')) {
        $missing = Copy-Json $override
        $missing.PSObject.Properties.Remove($field)
        Assert-Rejected (Invoke-Admission $knownFailure $baseRegistry $coverage $fixtures $missing) 'MISSING_REQUIRED_FIELD' "missing override.$field"
    }

    $broad = Copy-Json $override
    $broad.finding_id = '*'
    Assert-Rejected (Invoke-Admission $knownFailure $baseRegistry $coverage $fixtures $broad) 'OVERRIDE_SCOPE_MISMATCH' 'broad override'

    $stale = Copy-Json $override
    $stale.finding_id = 'finding-stale'
    Assert-Rejected (Invoke-Admission $knownFailure $baseRegistry $coverage $fixtures $stale) 'OVERRIDE_SCOPE_MISMATCH' 'stale override'

    $expired = Copy-Json $override
    $expired.expires_at = '2000-01-01T00:00:00Z'
    Assert-Rejected (Invoke-Admission $knownFailure $baseRegistry $coverage $fixtures $expired) 'OVERRIDE_EXPIRED' 'expired override'

    $digestMismatch = Copy-Json $override
    $digestMismatch.dossier_digest = '0' * 64
    Assert-Rejected (Invoke-Admission $knownFailure $baseRegistry $coverage $fixtures $digestMismatch) 'OVERRIDE_DIGEST_MISMATCH' 'digest-mismatched override'

    $dossierMismatch = Copy-Json $override
    $dossierMismatch.dossier_id = 'different-dossier'
    Assert-Rejected (Invoke-Admission $knownFailure $baseRegistry $coverage $fixtures $dossierMismatch) 'OVERRIDE_SCOPE_MISMATCH' 'dossier-mismatched override'

    $invalidDecision = Copy-Json $override
    $invalidDecision.decision = 'auto-admit-small-change'
    Assert-Rejected (Invoke-Admission $knownFailure $baseRegistry $coverage $fixtures $invalidDecision) 'INVALID_OWNER_DECISION' 'severity-based override'

    Write-Output 'PASS: owner override admission contract'
    exit 0
}

if ($Case -eq 'negative-fixtures') {
    $coverage = Get-Content -LiteralPath $coveragePath -Raw -Encoding utf8 | ConvertFrom-Json
    $fixtures = Get-Content -LiteralPath $fixturePath -Raw -Encoding utf8 | ConvertFrom-Json
    Assert-True ($fixtures.schema_version -eq 'x4-negative-fixtures.v1') 'Negative fixture schema is not versioned.'
    Assert-True (@($fixtures.fixtures).Count -eq 7) 'Exactly seven initial negative fixtures are required.'
    $orderedIds = @($fixtures.fixtures | Sort-Object id | ForEach-Object id)
    Assert-True (($orderedIds -join '|') -eq (@($fixtures.fixtures | ForEach-Object id) -join '|')) 'Negative fixtures must use deterministic ID order.'

    foreach ($fixture in @($fixtures.fixtures)) {
        Assert-True ($fixture.enabled -is [bool] -and $fixture.enabled) "Fixture $($fixture.id) is skipped."
        $dossier = Copy-Json $baseDossier
        $dimension = @($dossier.dimensions | Where-Object { $_.id -eq $fixture.dimension_id })
        Assert-True ($dimension.Count -eq 1) "Fixture $($fixture.id) names an unknown dimension."
        $dimension[0] | Add-Member -NotePropertyName finding_ids -NotePropertyValue @($fixture.finding_id)
        $dossier.findings = @([pscustomobject]@{
            id = $fixture.finding_id
            failure_class_id = $fixture.failure_class_id
            dimension_id = $fixture.dimension_id
            disposition = 'known-failure'
        })
        Assert-Rejected (Invoke-Admission $dossier $baseRegistry $coverage $fixtures) $fixture.expected_reason_code "fixture $($fixture.id)"
    }

    $missingRow = Copy-Json $coverage
    $missingRow.rows = @($missingRow.rows | Select-Object -Skip 1)
    Assert-Rejected (Invoke-Admission $baseDossier $baseRegistry $missingRow $fixtures) 'UNCOVERED_FAILURE_CLASS' 'missing coverage row'

    $missingFixture = Copy-Json $fixtures
    $missingFixture.fixtures = @($missingFixture.fixtures | Select-Object -Skip 1)
    Assert-Rejected (Invoke-Admission $baseDossier $baseRegistry $coverage $missingFixture) 'INVALID_FIXTURE_REFERENCE' 'missing fixture'

    $skippedFixture = Copy-Json $fixtures
    $skippedFixture.fixtures[0].enabled = $false
    Assert-Rejected (Invoke-Admission $baseDossier $baseRegistry $coverage $skippedFixture) 'SKIPPED_NEGATIVE_FIXTURE' 'skipped fixture'

    $mismatchedFixture = Copy-Json $fixtures
    $mismatchedFixture.fixtures[0].failure_class_id = 'native-binding-assumptions'
    Assert-Rejected (Invoke-Admission $baseDossier $baseRegistry $coverage $mismatchedFixture) 'MISMATCHED_FIXTURE' 'mismatched fixture'

    $duplicateFixture = Copy-Json $fixtures
    $duplicateFixture.fixtures = @($duplicateFixture.fixtures) + @(Copy-Json $duplicateFixture.fixtures[0])
    Assert-Rejected (Invoke-Admission $baseDossier $baseRegistry $coverage $duplicateFixture) 'DUPLICATE_ID' 'duplicate fixture'

    $passingFixture = Copy-Json $fixtures
    $passingFixture.fixtures[0].expected_reason_code = 'ADMISSIBLE'
    Assert-Rejected (Invoke-Admission $baseDossier $baseRegistry $coverage $passingFixture) 'PASSING_NEGATIVE_FIXTURE' 'passing negative fixture'

    $newRegistry = Copy-Json $baseRegistry
    $newClass = Copy-Json $newRegistry.failure_classes[0]
    $newClass.id = 'new-runtime-failure'
    $newClass.title = 'New runtime failure awaiting coverage'
    $newRegistry.failure_classes = @($newRegistry.failure_classes) + @($newClass)
    Assert-Rejected (Invoke-Admission $baseDossier $newRegistry $coverage $fixtures) 'UNCOVERED_FAILURE_CLASS' 'new registry row without coverage'

    Write-Output 'PASS: negative fixture coverage contract'
    exit 0
}

$complete = Invoke-Admission $baseDossier $baseRegistry
Assert-True ($complete.ExitCode -eq 0) "Complete dossier failed: $($complete.Output -join ' | ')"
Assert-True ($complete.Result.verdict -eq 'admissible') 'Complete dossier did not reach the admissible verdict.'
Assert-True (@($complete.Result.reason_codes).Count -eq 1 -and $complete.Result.reason_codes[0] -eq 'ADMISSIBLE') 'Complete dossier reason code is unstable.'

$unknownVersion = Copy-Json $baseDossier
$unknownVersion.schema_version = 'x4-integration-dossier.v999'
Assert-Rejected (Invoke-Admission $unknownVersion $baseRegistry) 'UNSUPPORTED_DOSSIER_SCHEMA' 'unknown dossier schema'

foreach ($field in @('schema_version', 'dossier_id', 'seam_id', 'evidence_sources', 'dimensions', 'findings')) {
    $missing = Copy-Json $baseDossier
    $missing.PSObject.Properties.Remove($field)
    Assert-Rejected (Invoke-Admission $missing $baseRegistry) 'MISSING_REQUIRED_FIELD' "missing dossier.$field"
}

foreach ($dimensionId in $requiredDimensions) {
    $missing = Copy-Json $baseDossier
    $missing.dimensions = @($missing.dimensions | Where-Object { $_.id -ne $dimensionId })
    Assert-Rejected (Invoke-Admission $missing $baseRegistry) 'MISSING_REQUIRED_DIMENSION' "missing dimension $dimensionId"
}

foreach ($field in @('id', 'status', 'primary_evidence_id', 'precedent_evidence_id')) {
    $missing = Copy-Json $baseDossier
    $missing.dimensions[0].PSObject.Properties.Remove($field)
    Assert-Rejected (Invoke-Admission $missing $baseRegistry) 'MISSING_REQUIRED_FIELD' "missing dimension.$field"
}

$missingPrimary = Copy-Json $baseDossier
$missingPrimary.dimensions[0].primary_evidence_id = 'missing-primary'
Assert-Rejected (Invoke-Admission $missingPrimary $baseRegistry) 'INVALID_EVIDENCE_REFERENCE' 'missing primary evidence'

$missingPrecedent = Copy-Json $baseDossier
$missingPrecedent.dimensions[0].precedent_evidence_id = 'missing-precedent'
Assert-Rejected (Invoke-Admission $missingPrecedent $baseRegistry) 'INVALID_EVIDENCE_REFERENCE' 'missing precedent evidence'

$sameEvidence = Copy-Json $baseDossier
$sameEvidence.dimensions[0].precedent_evidence_id = $sameEvidence.dimensions[0].primary_evidence_id
Assert-Rejected (Invoke-Admission $sameEvidence $baseRegistry) 'NON_INDEPENDENT_PROVENANCE' 'self-referential provenance'

$unverified = Copy-Json $baseDossier
$precedentId = $unverified.dimensions[0].precedent_evidence_id
($unverified.evidence_sources | Where-Object { $_.id -eq $precedentId }).verified = $false
Assert-Rejected (Invoke-Admission $unverified $baseRegistry) 'UNVERIFIED_PRODUCTION_PRECEDENT' 'unverified precedent'

$nonProduction = Copy-Json $baseDossier
($nonProduction.evidence_sources | Where-Object { $_.id -eq $precedentId }).production = $false
Assert-Rejected (Invoke-Admission $nonProduction $baseRegistry) 'UNVERIFIED_PRODUCTION_PRECEDENT' 'non-production precedent'

foreach ($status in @('UNKNOWN', 'CONFLICTING')) {
    $unresolved = Copy-Json $baseDossier
    $unresolved.dimensions[0].status = $status
    Assert-Rejected (Invoke-Admission $unresolved $baseRegistry) 'UNRESOLVED_EVIDENCE' "$status evidence"
}

$duplicateDimension = Copy-Json $baseDossier
$duplicateDimension.dimensions = @($duplicateDimension.dimensions) + @(Copy-Json $duplicateDimension.dimensions[0])
Assert-Rejected (Invoke-Admission $duplicateDimension $baseRegistry) 'DUPLICATE_ID' 'duplicate dimension ID'

$duplicateEvidence = Copy-Json $baseDossier
$duplicateEvidence.evidence_sources = @($duplicateEvidence.evidence_sources) + @(Copy-Json $duplicateEvidence.evidence_sources[0])
Assert-Rejected (Invoke-Admission $duplicateEvidence $baseRegistry) 'DUPLICATE_ID' 'duplicate evidence ID'

$overRows = Copy-Json $baseDossier
$overRows.findings = 1..65 | ForEach-Object {
    [pscustomobject]@{ id = "finding-$_"; failure_class_id = 'loader-mismatch'; disposition = 'resolved' }
}
Assert-Rejected (Invoke-Admission $overRows $baseRegistry) 'BOUND_EXCEEDED' 'excess finding rows'

$overBytes = Copy-Json $baseDossier
$overBytes.seam_id = 'x' * 70000
Assert-Rejected (Invoke-Admission $overBytes $baseRegistry) 'BOUND_EXCEEDED' 'excess dossier bytes'

Write-Output 'PASS: dossier admission contract'
