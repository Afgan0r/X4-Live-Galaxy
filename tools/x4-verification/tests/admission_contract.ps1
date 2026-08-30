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
$authorityModulePath = Join-Path $toolRoot 'local-attestation.psm1'
$newOverridePath = Join-Path $toolRoot 'new-owner-override.ps1'
$testAuthorityFixturePath = Join-Path $PSScriptRoot 'fixtures/test-owner-root-fixture.v1.json'
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

function Invoke-Admission($Dossier, $Registry, $Coverage = $null, $Fixtures = $null, $Override = $null, $Ledger = $null, $Matrix = $null, $PendingLedger = $null, [bool]$ValidationOnly = $true) {
    $tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("live-galaxy-admission-{0}" -f [guid]::NewGuid().ToString('N'))
    [void](New-Item -ItemType Directory -Path $tempRoot)
    try {
        $tempDossier = Join-Path $tempRoot 'dossier.json'
        $tempRegistry = Join-Path $tempRoot 'registry.json'
        $Dossier | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $tempDossier -Encoding utf8NoBOM
        $inputDigestBefore = (Get-FileHash -LiteralPath $tempDossier -Algorithm SHA256).Hash.ToLowerInvariant()
        $Registry | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $tempRegistry -Encoding utf8NoBOM
        $arguments = @('-NoProfile', '-File', $admissionPath, '-DossierPath', $tempDossier, '-RegistryPath', $tempRegistry)
        if ($ValidationOnly) { $arguments += '-ValidateFixture' }
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
        if ($null -ne $Matrix) {
            $tempMatrix = Join-Path $tempRoot 'matrix.json'
            $Matrix | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $tempMatrix -Encoding utf8NoBOM
            $arguments += @('-CandidateMatrixPath', $tempMatrix)
        }
        if ($null -ne $Ledger -and $null -ne $PendingLedger) {
            $resolvedLedger = Copy-Json $Ledger
            $resolvedPending = Copy-Json $PendingLedger
            $sourceDigests = @{
                dossier_digest = (Get-FileHash -LiteralPath $tempDossier -Algorithm SHA256).Hash.ToLowerInvariant()
                registry_digest = (Get-FileHash -LiteralPath $tempRegistry -Algorithm SHA256).Hash.ToLowerInvariant()
                coverage_digest = (Get-FileHash -LiteralPath $tempCoverage -Algorithm SHA256).Hash.ToLowerInvariant()
                matrix_digest = (Get-FileHash -LiteralPath $tempMatrix -Algorithm SHA256).Hash.ToLowerInvariant()
            }
            foreach ($candidate in @($resolvedLedger.candidates)) {
                $pending = @($resolvedPending.candidates | Where-Object { $_.id -eq $candidate.id })[0]
                foreach ($digestName in $sourceDigests.Keys) {
                    $original = $pending.identity_digests.$digestName
                    $pending.identity_digests.$digestName = $sourceDigests[$digestName]
                    if ($candidate.identity_digests.$digestName -eq $original) {
                        $candidate.identity_digests.$digestName = $sourceDigests[$digestName]
                    }
                }
            }
            $tempPending = Join-Path $tempRoot 'pending-ledger.json'
            $resolvedPending | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $tempPending -Encoding utf8NoBOM
            $arguments += @('-PendingLedgerPath', $tempPending)
            $tempLedger = Join-Path $tempRoot 'ledger.json'
            $resolvedLedger | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $tempLedger -Encoding utf8NoBOM
            $arguments += @('-SanitizedLedgerPath', $tempLedger)
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
    $requiredPaths += @($coveragePath, $fixturePath, $overrideContractPath, $authorityModulePath, $newOverridePath, $testAuthorityFixturePath)
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
    $completeLedger.evidence_classification = 'retained-runtime-evidence'
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

    $admissionDossier = Copy-Json $baseDossier
    $admissionDossier.seam_id = 'phase-05.1-runtime-discovery'
    $forged = Invoke-Admission $admissionDossier $baseRegistry $coverage $fixtures $null $completeLedger $matrix $pendingLedger $false
    Assert-Rejected $forged 'UNTRUSTED_EVIDENCE_SOURCE' 'hand-authored completed ledger'
    Write-Output 'PASS: hand-authored evidence-chain rejection contract'
    $retentionContractPath = Join-Path $PSScriptRoot 'evidence_retention_contract.ps1'
    $retentionOutput = @(& pwsh -NoProfile -File $retentionContractPath -Case retention-admission 2>&1)
    Assert-True ($LASTEXITCODE -eq 0) "Verified-locator contract failed: $($retentionOutput -join ' | ')"
    Assert-True (($retentionOutput -join "`n").Contains('PASS: retention-to-admission cryptographic core contract')) 'Verified locator did not reach the admission core.'
    Write-Output 'PASS: verified locator evidence-chain contract'
    Write-Output 'PASS: evidence-chain forgery rejection contract'
    exit 0
}

if ($Case -eq 'admission') {
    Import-Module $authorityModulePath -Force
    $coverage = Get-Content -LiteralPath $coveragePath -Raw -Encoding utf8 | ConvertFrom-Json
    $fixtures = Get-Content -LiteralPath $fixturePath -Raw -Encoding utf8 | ConvertFrom-Json
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

    $dossierBytes = [Text.Encoding]::UTF8.GetBytes(($knownFailure | ConvertTo-Json -Depth 32))
    $dossierDigest = ([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($dossierBytes))).ToLowerInvariant()
    $issuedAt = [DateTimeOffset]::UtcNow.AddMinutes(-1).ToString('yyyy-MM-ddTHH:mm:ssZ')
    $expiresAt = [DateTimeOffset]::UtcNow.AddDays(30).ToString('yyyy-MM-ddTHH:mm:ssZ')
    $payload = [pscustomobject][ordered]@{
        schema_version = 'x4-owner-override.v1'
        override_id = 'owner-override-test-exact'
        authority_purpose = 'owner-override'
        delegation_certificate_id = 'test-only-owner-override-delegation'
        dossier_id = $knownFailure.dossier_id
        dossier_digest = $dossierDigest
        finding_id = 'finding-small-loader-exception'
        owner_decision_id = 'owner-decision-test-exact'
        decision = 'accept-risk'
        rationale = 'TEST-ONLY exact acceptance for the isolated evaluator contract.'
        remaining_risk = 'TEST-ONLY loader mismatch remains isolated to this exact finding.'
        issued_at = $issuedAt
        expires_at = $expiresAt
        nonce = 'test-only-nonce-0001'
        signature_algorithm = 'ECDSA_P256_SHA256'
    }
    $signed = New-TestOnlyOwnerOverrideEnvelope -Payload $payload -FixturePath $testAuthorityFixturePath
    $verified = Test-TestOnlyExactOwnerOverride -Override $signed -FixturePath $testAuthorityFixturePath -ExpectedDossierId $knownFailure.dossier_id -ExpectedDossierDigest $dossierDigest -KnownFindingIds @('finding-small-loader-exception')
    Assert-True ($verified.status -eq 'OWNER_OVERRIDE_VERIFIED') 'Exact TEST-ONLY cryptographic/evaluator core did not pass.'
    Assert-True (($verified.overridden_finding_ids -join '|') -eq 'finding-small-loader-exception') 'Exact override changed the wrong finding.'
    $replayed = Test-TestOnlyExactOwnerOverride -Override $signed -FixturePath $testAuthorityFixturePath -ExpectedDossierId $knownFailure.dossier_id -ExpectedDossierDigest $dossierDigest -KnownFindingIds @('finding-small-loader-exception')
    Assert-True ($replayed.status -eq 'OWNER_OVERRIDE_VERIFIED') 'Unchanged exact decision was not idempotent.'

    $productionAttempt = Invoke-Admission $knownFailure $baseRegistry $coverage $fixtures $signed
    Assert-Rejected $productionAttempt 'OWNER_ROOT_UNCONFIGURED' 'TEST-ONLY root at production admission'

    foreach ($mutation in @(
        @{ field = 'finding_id'; value = 'finding-stale'; code = 'OVERRIDE_SCOPE_MISMATCH'; resign = $true },
        @{ field = 'finding_id'; value = '*'; code = 'INVALID_FIELD_VALUE'; resign = $true },
        @{ field = 'dossier_id'; value = 'different-dossier'; code = 'OVERRIDE_SCOPE_MISMATCH'; resign = $true },
        @{ field = 'dossier_digest'; value = ('0' * 64); code = 'OVERRIDE_DIGEST_MISMATCH'; resign = $true },
        @{ field = 'decision'; value = 'auto-admit-small-change'; code = 'INVALID_OWNER_DECISION'; resign = $true },
        @{ field = 'rationale'; value = 'altered rationale'; code = 'OWNER_OVERRIDE_PAYLOAD_DIGEST_MISMATCH' },
        @{ field = 'remaining_risk'; value = 'altered risk'; code = 'OWNER_OVERRIDE_PAYLOAD_DIGEST_MISMATCH' },
        @{ field = 'expires_at'; value = [DateTimeOffset]::UtcNow.AddDays(60).ToString('yyyy-MM-ddTHH:mm:ssZ'); code = 'OWNER_OVERRIDE_PAYLOAD_DIGEST_MISMATCH' },
        @{ field = 'nonce'; value = 'test-only-nonce-0002'; code = 'OWNER_OVERRIDE_PAYLOAD_DIGEST_MISMATCH' }
    )) {
        $altered = Copy-Json $signed
        $altered.($mutation.field) = $mutation.value
        if ($mutation.ContainsKey('resign')) {
            $resignedPayload = [ordered]@{}
            foreach ($field in @('schema_version', 'override_id', 'authority_purpose', 'delegation_certificate_id', 'dossier_id', 'dossier_digest', 'finding_id', 'owner_decision_id', 'decision', 'rationale', 'remaining_risk', 'issued_at', 'expires_at', 'nonce', 'signature_algorithm')) {
                $resignedPayload[$field] = $altered.$field
            }
            $altered = New-TestOnlyOwnerOverrideEnvelope -Payload ([pscustomobject]$resignedPayload) -FixturePath $testAuthorityFixturePath
        }
        $result = Test-TestOnlyExactOwnerOverride -Override $altered -FixturePath $testAuthorityFixturePath -ExpectedDossierId $knownFailure.dossier_id -ExpectedDossierDigest $dossierDigest -KnownFindingIds @('finding-small-loader-exception')
        Assert-True ($result.status -eq $mutation.code) "Altered $($mutation.field) returned $($result.status), expected $($mutation.code)."
    }

    $transplant = Test-TestOnlyExactOwnerOverride -Override $signed -FixturePath $testAuthorityFixturePath -ExpectedDossierId 'different-dossier' -ExpectedDossierDigest $dossierDigest -KnownFindingIds @('finding-small-loader-exception')
    Assert-True ($transplant.status -eq 'OVERRIDE_SCOPE_MISMATCH') 'Signature transplant to another dossier was accepted.'

    foreach ($missingField in @('schema_version', 'authority_purpose', 'dossier_id', 'finding_id', 'decision', 'rationale', 'remaining_risk', 'issued_at', 'expires_at', 'nonce', 'payload_digest', 'signature_base64')) {
        $missing = Copy-Json $signed
        $missing.PSObject.Properties.Remove($missingField)
        $missingResult = Test-TestOnlyExactOwnerOverride -Override $missing -FixturePath $testAuthorityFixturePath -ExpectedDossierId $knownFailure.dossier_id -ExpectedDossierDigest $dossierDigest -KnownFindingIds @('finding-small-loader-exception')
        Assert-True ($missingResult.status -eq 'MISSING_REQUIRED_FIELD') "Missing $missingField did not fail closed."
    }

    $expiredPayload = [ordered]@{}
    foreach ($field in @('schema_version', 'override_id', 'authority_purpose', 'delegation_certificate_id', 'dossier_id', 'dossier_digest', 'finding_id', 'owner_decision_id', 'decision', 'rationale', 'remaining_risk', 'issued_at', 'expires_at', 'nonce', 'signature_algorithm')) {
        $expiredPayload[$field] = $signed.$field
    }
    $expiredPayload.issued_at = '1999-12-01T00:00:00Z'
    $expiredPayload.expires_at = '2000-01-01T00:00:00Z'
    $expiredSigned = New-TestOnlyOwnerOverrideEnvelope -Payload ([pscustomobject]$expiredPayload) -FixturePath $testAuthorityFixturePath
    $expiredResult = Test-TestOnlyExactOwnerOverride -Override $expiredSigned -FixturePath $testAuthorityFixturePath -ExpectedDossierId $knownFailure.dossier_id -ExpectedDossierDigest $dossierDigest -KnownFindingIds @('finding-small-loader-exception')
    Assert-True ($expiredResult.status -eq 'OVERRIDE_EXPIRED') 'Expired exact override was accepted.'

    $signerTemp = Join-Path ([IO.Path]::GetTempPath()) ("live-galaxy-owner-cli-{0}" -f [guid]::NewGuid().ToString('N'))
    [void](New-Item -ItemType Directory -Path $signerTemp)
    try {
        $dossierFile = Join-Path $signerTemp 'dossier.json'
        $outputFile = Join-Path $signerTemp 'override.json'
        $knownFailure | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $dossierFile -Encoding utf8NoBOM
        $cliOutput = & pwsh -NoProfile -File $newOverridePath -DossierPath $dossierFile -FindingId 'finding-small-loader-exception' -OwnerDecisionId 'owner-decision-production-probe' -Rationale 'Bounded production authority availability probe.' -RemainingRisk 'Known loader mismatch remains bounded to this finding.' -ExpiresAt $expiresAt -OutputPath $outputFile 2>&1
        Assert-True ($LASTEXITCODE -ne 0) 'Unconfigured production signer unexpectedly succeeded.'
        $cliResult = @($cliOutput | ForEach-Object { $_.ToString() } | Where-Object { $_.StartsWith('{') })[-1] | ConvertFrom-Json
        Assert-True ($cliResult.status -eq 'OWNER_ROOT_UNCONFIGURED') "Production signer returned $($cliResult.status)."
        Assert-True (-not (Test-Path -LiteralPath $outputFile)) 'Unconfigured production signer wrote an override.'
    }
    finally { Remove-Item -LiteralPath $signerTemp -Recurse -Force }

    $newOverrideCommand = Get-Command $newOverridePath
    foreach ($forbiddenParameter in @('AnchorPath', 'RootPath', 'PublicKeyPath', 'KeyName', 'Sid', 'TestMode', 'VerifierPath')) {
        Assert-True ($newOverrideCommand.Parameters.Keys -notcontains $forbiddenParameter) "Production signer exposes forbidden $forbiddenParameter parameter."
    }

    Write-Output 'PASS: authenticated exact owner override contract'
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

    $fixtureValidation = Invoke-Admission $baseDossier $baseRegistry $coverage $fixtures
    Assert-True ($fixtureValidation.ExitCode -eq 0) 'Executable negative fixture bundle did not validate.'
    $resultMap = @{}
    foreach ($result in @($fixtureValidation.Result.negative_fixture_results)) { $resultMap[$result.id] = $result.reason_code }
    foreach ($fixture in @($fixtures.fixtures)) {
        Assert-True ($fixture.enabled -is [bool] -and $fixture.enabled) "Fixture $($fixture.id) is skipped."
        Assert-True ($resultMap[$fixture.id] -eq $fixture.expected_reason_code) "Fixture $($fixture.id) did not execute its class-specific detector."
    }

    $passingMutation = Copy-Json $fixtures
    $passingMutation.fixtures[0].detector_input.packaged_entrypoint = $passingMutation.fixtures[0].detector_input.registered_entrypoint
    Assert-Rejected (Invoke-Admission $baseDossier $baseRegistry $coverage $passingMutation) 'NEGATIVE_FIXTURE_DID_NOT_FAIL' 'loader fixture without loader defect'

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
    $passingFixture.fixtures[0].expected_reason_code = 'loader-mismatch-wrong-code'
    Assert-Rejected (Invoke-Admission $baseDossier $baseRegistry $coverage $passingFixture) 'NEGATIVE_FIXTURE_REASON_MISMATCH' 'mismatched detector reason'

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
Assert-True ($complete.Result.verdict -eq 'validation-passed') 'Complete dossier fixture did not reach validation-passed.'
Assert-True (@($complete.Result.reason_codes).Count -eq 1 -and $complete.Result.reason_codes[0] -eq 'VALIDATION_PASSED') 'Complete dossier validation reason code is unstable.'

$directAdmission = Invoke-Admission $baseDossier $baseRegistry $null $null $null $null $null $null $false
Assert-Rejected $directAdmission 'VALIDATION_FIXTURE_NOT_ADMISSIBLE' 'validation fixture in admission mode'

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
