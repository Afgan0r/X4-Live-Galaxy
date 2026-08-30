[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string]$DossierPath,
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string]$RegistryPath,
    [string]$CoveragePath,
    [string]$FixturePath,
    [string]$OverridePath,
    [string]$SanitizedLedgerPath,
    [string]$VerifiedLocatorPath,
    [string]$PendingLedgerPath,
    [string]$CandidateMatrixPath,
    [switch]$ValidateFixture
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'local-attestation.psm1') -Force
$retentionVerifierPath = Join-Path $PSScriptRoot 'retain-evidence.ps1'

$maxInputBytes = 65536
$maxEvidenceSources = 64
$maxDimensions = 32
$maxFindings = 64
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
$requiredFailureClasses = @(
    'loader-mismatch',
    'native-binding-assumptions',
    'native-canonical-identity-mismatch',
    'invented-unmeasured-bounds',
    'partial-results-represented-complete',
    'permissive-local-harness',
    'isolated-call-shape-research'
)

$script:failureCode = 'INTERNAL_VALIDATION_ERROR'
$script:dossierId = 'unparsed'
$script:dossierDigest = $null
$script:validationContext = 'startup'
$script:overriddenFindingIds = @()
$script:negativeFixtureResults = @()

function Fail([string]$Code) {
    $script:failureCode = $Code
    throw [System.InvalidOperationException]::new($Code)
}

function Require-Property($Value, [string]$Name) {
    if ($null -eq $Value -or $Value.PSObject.Properties.Name -notcontains $Name) {
        Fail 'MISSING_REQUIRED_FIELD'
    }
    return $Value.$Name
}

function Require-Text($Value) {
    if ($Value -isnot [string] -or [string]::IsNullOrWhiteSpace($Value) -or $Value.Length -gt 256) {
        Fail 'INVALID_FIELD_VALUE'
    }
}

function Require-Id($Value) {
    Require-Text $Value
    if ($Value -notmatch '^[a-z0-9][a-z0-9._-]{0,127}$') {
        Fail 'INVALID_FIELD_VALUE'
    }
}

function Require-Bool($Value) {
    if ($Value -isnot [bool]) {
        Fail 'INVALID_BOOLEAN'
    }
}

function Require-Digest($Value) {
    if ($Value -isnot [string] -or $Value -notmatch '^[a-f0-9]{64}$') {
        Fail 'EVIDENCE_CHAIN_INCOMPLETE'
    }
}

function Require-Array($Value, [int]$Maximum) {
    if ($null -eq $Value -or $Value -is [string]) {
        Fail 'INVALID_FIELD_VALUE'
    }
    $items = @($Value)
    if ($items.Count -gt $Maximum) {
        Fail 'BOUND_EXCEEDED'
    }
    return $items
}

function Assert-UniqueIds($Items) {
    $ids = @($Items | ForEach-Object { [string](Require-Property $_ 'id') })
    if (@($ids | Sort-Object -Unique).Count -ne $ids.Count) {
        Fail 'DUPLICATE_ID'
    }
}

function Require-ExactProperties($Value, [string[]]$Names) {
    $actual = @($Value.PSObject.Properties.Name | Sort-Object)
    $expected = @($Names | Sort-Object)
    if (($actual -join '|') -ne ($expected -join '|')) {
        if (@($expected | Where-Object { $actual -notcontains $_ }).Count -gt 0) {
            Fail 'EVIDENCE_CHAIN_INCOMPLETE'
        }
        Fail 'UNSANITIZED_LEDGER_FIELD'
    }
}

function Read-BoundedJson([string]$Path, [string]$SchemaVersion, [string]$SchemaFailureCode) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Fail 'MISSING_INPUT'
    }
    $bytes = [System.IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Path).Path)
    if ($bytes.Length -gt $maxInputBytes) {
        Fail 'BOUND_EXCEEDED'
    }
    try {
        $value = [System.Text.Encoding]::UTF8.GetString($bytes) | ConvertFrom-Json
    }
    catch {
        Fail 'MALFORMED_JSON'
    }
    if ((Require-Property $value 'schema_version') -ne $SchemaVersion) {
        Fail $SchemaFailureCode
    }
    return [pscustomobject]@{ Value = $value; Bytes = $bytes }
}

function Get-Sha256Hex([byte[]]$Bytes) {
    $hash = [System.Security.Cryptography.SHA256]::HashData($Bytes)
    return ([System.Convert]::ToHexString($hash)).ToLowerInvariant()
}

function Get-EvidenceMap($Document) {
    $sources = Require-Array (Require-Property $Document 'evidence_sources') $maxEvidenceSources
    Assert-UniqueIds $sources
    $map = @{}
    foreach ($source in $sources) {
        $id = Require-Property $source 'id'
        Require-Id $id
        $class = Require-Property $source 'class'
        if ($class -notin @('primary-source', 'working-production-precedent')) {
            Fail 'INVALID_PROVENANCE_CLASS'
        }
        Require-Text (Require-Property $source 'kind')
        Require-Text (Require-Property $source 'reference')
        Require-Text (Require-Property $source 'verification_scope')
        Require-Bool (Require-Property $source 'verified')
        Require-Bool (Require-Property $source 'production')
        $map[$id] = $source
    }
    return $map
}

function Assert-Provenance($Item, $EvidenceMap) {
    $primaryId = Require-Property $Item 'primary_evidence_id'
    $precedentId = Require-Property $Item 'precedent_evidence_id'
    Require-Id $primaryId
    Require-Id $precedentId
    if ($primaryId -eq $precedentId) {
        Fail 'NON_INDEPENDENT_PROVENANCE'
    }
    if (-not $EvidenceMap.ContainsKey($primaryId) -or -not $EvidenceMap.ContainsKey($precedentId)) {
        Fail 'INVALID_EVIDENCE_REFERENCE'
    }
    if ($EvidenceMap[$primaryId].class -ne 'primary-source' -or
        $EvidenceMap[$precedentId].class -ne 'working-production-precedent') {
        Fail 'INVALID_PROVENANCE_CLASS'
    }
    if (-not $EvidenceMap[$precedentId].verified -or -not $EvidenceMap[$precedentId].production) {
        Fail 'UNVERIFIED_PRODUCTION_PRECEDENT'
    }
}

function Test-Registry($Registry) {
    Require-Id (Require-Property $Registry 'registry_id')
    $evidenceMap = Get-EvidenceMap $Registry
    $classes = Require-Array (Require-Property $Registry 'failure_classes') 32
    Assert-UniqueIds $classes
    $ids = @()
    foreach ($failureClass in $classes) {
        $id = Require-Property $failureClass 'id'
        Require-Id $id
        Require-Text (Require-Property $failureClass 'title')
        Assert-Provenance $failureClass $evidenceMap
        $ids += $id
    }
    foreach ($requiredClass in $requiredFailureClasses) {
        if ($ids -notcontains $requiredClass) {
            Fail 'INVALID_FAILURE_REGISTRY'
        }
    }
    if ($ids.Count -lt $requiredFailureClasses.Count) {
        Fail 'INVALID_FAILURE_REGISTRY'
    }
    return [pscustomobject]@{ Ids = @($ids); EvidenceMap = $evidenceMap; Classes = @($classes) }
}

function Test-FixtureBundle($Bundle, [string[]]$FailureClassIds) {
    Require-Id (Require-Property $Bundle 'fixture_bundle_id')
    $maxExecutionMs = Require-Property $Bundle 'max_execution_ms'
    if ($maxExecutionMs -isnot [long] -and $maxExecutionMs -isnot [int]) {
        Fail 'INVALID_FIELD_VALUE'
    }
    if ($maxExecutionMs -lt 1 -or $maxExecutionMs -gt 10000) {
        Fail 'BOUND_EXCEEDED'
    }
    $fixtures = Require-Array (Require-Property $Bundle 'fixtures') 32
    Assert-UniqueIds $fixtures
    $fixtureIds = @($fixtures | ForEach-Object { [string]$_.id })
    if (($fixtureIds -join '|') -ne (@($fixtureIds | Sort-Object) -join '|')) {
        Fail 'NON_DETERMINISTIC_ORDER'
    }
    $map = @{}
    foreach ($fixture in $fixtures) {
        $id = Require-Property $fixture 'id'
        Require-Id $id
        $failureClassId = Require-Property $fixture 'failure_class_id'
        if ($FailureClassIds -notcontains $failureClassId) {
            Fail 'INVALID_EVIDENCE_REFERENCE'
        }
        $dimensionId = Require-Property $fixture 'dimension_id'
        if ($requiredDimensions -notcontains $dimensionId) {
            Fail 'INVALID_EVIDENCE_REFERENCE'
        }
        Require-Bool (Require-Property $fixture 'enabled')
        if (-not $fixture.enabled) {
            Fail 'SKIPPED_NEGATIVE_FIXTURE'
        }
        $expectedReason = Require-Property $fixture 'expected_reason_code'
        Require-Id $expectedReason
        $detectedReason = Invoke-NegativeFixtureDetector $fixture
        if ($detectedReason -ne $expectedReason) { Fail 'NEGATIVE_FIXTURE_REASON_MISMATCH' }
        $script:negativeFixtureResults += [ordered]@{ id = $id; reason_code = $detectedReason }
        $map[$id] = $fixture
    }
    return $map
}

function Invoke-NegativeFixtureDetector($Fixture) {
    $detector = Require-Property $Fixture 'detector'
    $input = Require-Property $Fixture 'detector_input'
    switch ($detector) {
        'package-registration' {
            if ($input.registered_entrypoint -cne $input.packaged_entrypoint) { return 'loader-mismatch-detected' }
        }
        'native-binding' {
            if ($input.binding_acquired -ne $true -or $input.call_verified -ne $true) { return 'native-binding-unverified' }
        }
        'identity-closure' {
            if ($input.native_id -ne $input.canonical_id -or $input.owner_matches -ne $true) { return 'identity-mismatch-detected' }
        }
        'bound-provenance' {
            if ([string]::IsNullOrWhiteSpace([string]$input.evidence_id) -or $input.measured -ne $true) { return 'bound-provenance-missing' }
        }
        'atomic-completeness' {
            if ($input.completion_claimed -eq $true -and $input.valid_count -ne $input.expected_count) { return 'partial-completion-detected' }
        }
        'package-resolution' {
            if ($input.test_resolves -eq $true -and $input.production_resolves -ne $true) { return 'permissive-harness-dependency' }
        }
        'integration-context' {
            $required = @('loader-registration', 'module-resolution', 'lifecycle-thread', 'native-canonical-identity', 'failure-partial-completeness', 'volume-performance')
            $present = @($input.context_dimensions)
            if ($input.call_shape_present -eq $true -and @($required | Where-Object { $present -notcontains $_ }).Count -gt 0) {
                return 'isolated-call-context-incomplete'
            }
        }
        default { Fail 'INVALID_FIXTURE_DETECTOR' }
    }
    Fail 'NEGATIVE_FIXTURE_DID_NOT_FAIL'
}

function Test-Coverage($Coverage, $RegistryInfo, $FixtureMap) {
    Require-Id (Require-Property $Coverage 'coverage_id')
    if ((Require-Property $Coverage 'registry_id') -ne 'live-galaxy-known-x4-failures') {
        Fail 'INVALID_EVIDENCE_REFERENCE'
    }
    $rows = Require-Array (Require-Property $Coverage 'rows') 32
    Assert-UniqueIds $rows
    $rowMap = @{}
    foreach ($row in $rows) {
        $id = Require-Property $row 'id'
        Require-Id $id
        $failureClassId = Require-Property $row 'failure_class_id'
        if ($id -ne $failureClassId) {
            Fail 'MISMATCHED_COVERAGE_ROW'
        }
        $registryClass = @($RegistryInfo.Classes | Where-Object { $_.id -eq $failureClassId })
        if ($registryClass.Count -ne 1) {
            Fail 'INVALID_EVIDENCE_REFERENCE'
        }
        if ((Require-Property $row 'primary_evidence_id') -ne $registryClass[0].primary_evidence_id -or
            (Require-Property $row 'precedent_evidence_id') -ne $registryClass[0].precedent_evidence_id) {
            Fail 'MISMATCHED_COVERAGE_ROW'
        }
        $status = Require-Property $row 'status'
        if ($status -notin @('covered', 'not-applicable')) {
            Fail 'INVALID_FIELD_VALUE'
        }
        Require-Text (Require-Property $row 'executable_check')
        $null = Require-Property $row 'fixture_ids'
        if ($row.fixture_ids -is [string]) {
            Fail 'INVALID_FIELD_VALUE'
        }
        $fixtureIds = @($row.fixture_ids)
        if ($fixtureIds.Count -gt 8) {
            Fail 'BOUND_EXCEEDED'
        }
        if ($status -eq 'covered' -and $fixtureIds.Count -eq 0) {
            Fail 'INVALID_FIXTURE_REFERENCE'
        }
        if ($status -eq 'not-applicable') {
            Require-Text (Require-Property $row 'justification')
            if ($fixtureIds.Count -ne 0) {
                Fail 'MISMATCHED_COVERAGE_ROW'
            }
        }
        foreach ($fixtureId in $fixtureIds) {
            Require-Id $fixtureId
            if (-not $FixtureMap.ContainsKey($fixtureId)) {
                Fail 'INVALID_FIXTURE_REFERENCE'
            }
            if ($FixtureMap[$fixtureId].failure_class_id -ne $failureClassId) {
                Fail 'MISMATCHED_FIXTURE'
            }
        }
        $rowMap[$failureClassId] = $row
    }
    foreach ($failureClassId in $RegistryInfo.Ids) {
        if (-not $rowMap.ContainsKey($failureClassId)) {
            Fail 'UNCOVERED_FAILURE_CLASS'
        }
    }
    if ($rowMap.Count -ne $RegistryInfo.Ids.Count) {
        Fail 'UNCOVERED_FAILURE_CLASS'
    }
}

function Test-Dossier($Dossier, [string[]]$FailureClassIds, [bool]$AllowRuntimeResolution) {
    $script:dossierId = Require-Property $Dossier 'dossier_id'
    Require-Id $script:dossierId
    Require-Id (Require-Property $Dossier 'seam_id')
    $evidenceMap = Get-EvidenceMap $Dossier
    $dimensions = Require-Array (Require-Property $Dossier 'dimensions') $maxDimensions
    Assert-UniqueIds $dimensions
    $dimensionIds = @()
    $dimensionFindingRefs = @{}
    foreach ($dimension in $dimensions) {
        $id = Require-Property $dimension 'id'
        Require-Id $id
        $status = Require-Property $dimension 'status'
        if ($status -notin @('EVIDENCED', 'CONFLICTING', 'UNKNOWN')) {
            Fail 'INVALID_EVIDENCE_STATUS'
        }
        Assert-Provenance $dimension $evidenceMap
        if ($status -ne 'EVIDENCED' -and -not $AllowRuntimeResolution) {
            Fail 'UNRESOLVED_EVIDENCE'
        }
        $findingIds = @()
        if ($dimension.PSObject.Properties.Name -contains 'finding_ids') {
            if ($dimension.finding_ids -is [string]) {
                Fail 'INVALID_FIELD_VALUE'
            }
            $findingIds = @($dimension.finding_ids)
            if ($findingIds.Count -gt 8) {
                Fail 'BOUND_EXCEEDED'
            }
            foreach ($findingId in $findingIds) {
                Require-Id $findingId
            }
            if (@($findingIds | Sort-Object -Unique).Count -ne $findingIds.Count) {
                Fail 'DUPLICATE_ID'
            }
        }
        $dimensionFindingRefs[$id] = $findingIds
        $dimensionIds += $id
    }
    if ((@($dimensionIds | Sort-Object) -join '|') -ne (@($requiredDimensions | Sort-Object) -join '|')) {
        Fail 'MISSING_REQUIRED_DIMENSION'
    }

    $null = Require-Property $Dossier 'findings'
    $findings = @($Dossier.findings)
    if ($findings.Count -gt $maxFindings) {
        Fail 'BOUND_EXCEEDED'
    }
    Assert-UniqueIds $findings
    $findingMap = @{}
    $knownFindings = @()
    foreach ($finding in $findings) {
        $findingId = Require-Property $finding 'id'
        Require-Id $findingId
        $failureClassId = Require-Property $finding 'failure_class_id'
        if ($FailureClassIds -notcontains $failureClassId) {
            Fail 'INVALID_EVIDENCE_REFERENCE'
        }
        $dimensionId = Require-Property $finding 'dimension_id'
        if ($requiredDimensions -notcontains $dimensionId) {
            Fail 'INVALID_EVIDENCE_REFERENCE'
        }
        $disposition = Require-Property $finding 'disposition'
        if ($disposition -notin @('resolved', 'known-failure')) {
            Fail 'INVALID_FIELD_VALUE'
        }
        if ($disposition -eq 'known-failure') {
            $knownFindings += $finding
        }
        $findingMap[$findingId] = $finding
    }
    foreach ($dimensionId in $dimensionFindingRefs.Keys) {
        foreach ($findingId in $dimensionFindingRefs[$dimensionId]) {
            if (-not $findingMap.ContainsKey($findingId) -or $findingMap[$findingId].dimension_id -ne $dimensionId) {
                Fail 'INVALID_EVIDENCE_REFERENCE'
            }
        }
    }
    foreach ($finding in $findings) {
        $references = @($dimensionFindingRefs.Values | ForEach-Object { @($_) } | Where-Object { $_ -eq $finding.id })
        if ($references.Count -ne 1) {
            Fail 'INVALID_EVIDENCE_REFERENCE'
        }
    }
    return @($knownFindings)
}

function Test-OwnerOverride($Override, $KnownFindings) {
    $script:validationContext = 'override-required-fields'
    foreach ($field in @('override_id', 'authority_purpose', 'delegation_certificate_id', 'dossier_id', 'dossier_digest', 'finding_id', 'owner_decision_id', 'decision', 'rationale', 'remaining_risk', 'issued_at', 'expires_at', 'nonce', 'signature_algorithm', 'payload_digest', 'signature_base64')) {
        $null = Require-Property $Override $field
    }
    $script:validationContext = 'override-identifiers'
    Require-Id $Override.override_id
    Require-Id $Override.owner_decision_id
    $script:validationContext = 'override-rationale'
    Require-Text $Override.rationale
    Require-Text $Override.remaining_risk
    $script:validationContext = 'override-scope'
    if ($Override.dossier_id -ne $script:dossierId) {
        Fail 'OVERRIDE_SCOPE_MISMATCH'
    }
    if ($Override.dossier_digest -notmatch '^[a-f0-9]{64}$' -or $Override.dossier_digest -ne $script:dossierDigest) {
        Fail 'OVERRIDE_DIGEST_MISMATCH'
    }
    $matching = @($KnownFindings | Where-Object { $_.id -eq $Override.finding_id })
    if ($matching.Count -ne 1) {
        Fail 'OVERRIDE_SCOPE_MISMATCH'
    }
    if ($Override.decision -ne 'accept-risk') {
        Fail 'INVALID_OWNER_DECISION'
    }
    $script:validationContext = 'override-expiry'
    try {
        if ($Override.expires_at -is [DateTime]) {
            $expiry = [DateTimeOffset]::new($Override.expires_at.ToUniversalTime())
        }
        elseif ($Override.expires_at -is [DateTimeOffset]) {
            $expiry = $Override.expires_at.ToUniversalTime()
        }
        else {
            $expiry = [DateTimeOffset]::ParseExact(
                $Override.expires_at,
                'yyyy-MM-ddTHH:mm:ssZ',
                [System.Globalization.CultureInfo]::InvariantCulture,
                [System.Globalization.DateTimeStyles]::AssumeUniversal
            )
        }
    }
    catch {
        Fail 'INVALID_FIELD_VALUE'
    }
    $now = [DateTimeOffset]::UtcNow
    if ($expiry -le $now) {
        Fail 'OVERRIDE_EXPIRED'
    }
    if ($expiry -gt $now.AddDays(90)) {
        Fail 'OVERRIDE_EXPIRY_OUT_OF_RANGE'
    }
    return @($Override.finding_id)
}

function Test-EvidenceChain($Ledger, $PendingLedger, $Matrix, $ExpectedDigests) {
    $script:validationContext = 'evidence-chain-structure'
    if ($Ledger.schema_version -eq 'sanitized-ledger.v1') {
        $ledgerFields = @(
            'schema_version', 'ledger_id', 'run_id', 'build_id', 'group_id',
            'evidence_classification', 'runtime_evidence_schema_version',
            'build_manifest_schema_version', 'verdict', 'retention_disposition',
            'identity_digests', 'candidates'
        )
        Require-ExactProperties $Ledger $ledgerFields
        if ($Ledger.verdict -ne 'retained' -or $Ledger.retention_disposition -ne 'retained' -or
            $Ledger.evidence_classification -notin @('authenticated-local-contract', 'observed-in-X4')) {
            Fail 'RETENTION_INCOMPLETE'
        }
        $matrixCandidates = Require-Array (Require-Property $Matrix 'candidates') 7
        $pendingCandidates = Require-Array (Require-Property $PendingLedger 'candidates') 7
        $retainedCandidates = Require-Array (Require-Property $Ledger 'candidates') 7
        if ($matrixCandidates.Count -ne 7 -or $pendingCandidates.Count -ne 7 -or $retainedCandidates.Count -lt 1) {
            Fail 'EVIDENCE_CHAIN_INCOMPLETE'
        }
        $digests = Require-Property $Ledger 'identity_digests'
        $digestFields = @(
            'dossier_digest', 'registry_digest', 'coverage_digest', 'matrix_digest',
            'build_profile_digest', 'package_conformance_digest',
            'runtime_evidence_schema_digest', 'build_manifest_digest',
            'evidence_digest', 'producer_attestation_digest', 'run_digest', 'locator_digest'
        )
        Require-ExactProperties $digests $digestFields
        foreach ($name in $digestFields) { Require-Digest (Require-Property $digests $name) }
        if ($digests.dossier_digest -ne $ExpectedDigests.dossier_digest -or
            $digests.registry_digest -ne $ExpectedDigests.registry_digest -or
            $digests.coverage_digest -ne $ExpectedDigests.coverage_digest -or
            $digests.matrix_digest -ne $ExpectedDigests.matrix_digest) {
            Fail 'IDENTITY_CHAIN_MISMATCH'
        }
        $expectedIds = @($matrixCandidates | Where-Object { $_.build_profile_digest -eq $digests.build_profile_digest } | ForEach-Object id | Sort-Object)
        $actualIds = @($retainedCandidates | ForEach-Object candidate_id | Sort-Object)
        if ($expectedIds.Count -lt 1 -or ($actualIds -join '|') -ne ($expectedIds -join '|')) {
            Fail 'IDENTITY_CHAIN_MISMATCH'
        }
        foreach ($candidate in $retainedCandidates) {
            Require-ExactProperties $candidate @('candidate_id', 'execution_verdict', 'contract_verdict', 'effect_verdict', 'disposition')
            if ($candidate.execution_verdict -ne 'pass' -or $candidate.contract_verdict -ne 'pass' -or
                $candidate.effect_verdict -ne 'pass' -or $candidate.disposition -ne 'retain') {
                Fail 'FAILED_RUNTIME_VERDICT'
            }
            $pending = @($pendingCandidates | Where-Object { $_.id -eq $candidate.candidate_id })
            if ($pending.Count -ne 1 -or $pending[0].status -ne 'runtime-pending') { Fail 'IDENTITY_CHAIN_MISMATCH' }
        }
        return $Ledger.evidence_classification
    }
    $ledgerFields = @('schema_version', 'ledger_id', 'status', 'evidence_classification', 'candidates')
    Require-ExactProperties $Ledger $ledgerFields
    Require-ExactProperties $PendingLedger $ledgerFields
    Require-Id (Require-Property $Ledger 'ledger_id')
    Require-Id (Require-Property $PendingLedger 'ledger_id')
    if ((Require-Property $PendingLedger 'status') -ne 'runtime-pending') {
        Fail 'INVALID_FIELD_VALUE'
    }
    if ((Require-Property $Ledger 'status') -ne 'runtime-complete') {
        Fail 'RETENTION_INCOMPLETE'
    }
    if ((Require-Property $PendingLedger 'evidence_classification') -ne 'scaffold-only' -or
        (Require-Property $Ledger 'evidence_classification') -ne 'retained-runtime-evidence') {
        Fail 'RETENTION_INCOMPLETE'
    }
    Require-Id (Require-Property $Matrix 'matrix_id')
    $matrixCandidates = Require-Array (Require-Property $Matrix 'candidates') 7
    $pendingCandidates = Require-Array (Require-Property $PendingLedger 'candidates') 7
    $ledgerCandidates = Require-Array (Require-Property $Ledger 'candidates') 7
    if ($matrixCandidates.Count -ne 7 -or $pendingCandidates.Count -ne 7 -or $ledgerCandidates.Count -ne 7) {
        Fail 'EVIDENCE_CHAIN_INCOMPLETE'
    }
    Assert-UniqueIds $matrixCandidates
    Assert-UniqueIds $pendingCandidates
    Assert-UniqueIds $ledgerCandidates

    foreach ($matrixCandidate in $matrixCandidates) {
        if ((Require-Property $matrixCandidate 'source_action_only') -isnot [bool] -or $matrixCandidate.source_action_only) {
            Fail 'SOURCE_ACTION_DEFECT'
        }
    }

    $staticDigestNames = @(
        'dossier_digest',
        'registry_digest',
        'coverage_digest',
        'matrix_digest',
        'build_profile_digest',
        'package_conformance_digest',
        'runtime_evidence_schema_digest',
        'build_manifest_digest'
    )
    $pendingBoundDigestNames = @(
        'dossier_digest',
        'registry_digest',
        'coverage_digest',
        'matrix_digest',
        'build_profile_digest',
        'runtime_evidence_schema_digest'
    )
    foreach ($matrixCandidate in $matrixCandidates) {
        $id = Require-Property $matrixCandidate 'id'
        Require-Id $id
        $pending = @($pendingCandidates | Where-Object { $_.id -eq $id })
        $candidate = @($ledgerCandidates | Where-Object { $_.id -eq $id })
        if ($pending.Count -ne 1 -or $candidate.Count -ne 1) {
            Fail 'EVIDENCE_CHAIN_INCOMPLETE'
        }
        $pending = $pending[0]
        $candidate = $candidate[0]
        $candidateFields = @(
            'id', 'status', 'disposition', 'execution_verdict',
            'contract_verdict', 'effect_verdict', 'expected_effect_id',
            'actual_effect_id', 'run_id', 'evidence_requirement_ids',
            'stop_condition_ids', 'identity_digests'
        )
        Require-ExactProperties $pending $candidateFields
        Require-ExactProperties $candidate $candidateFields
        if ((Require-Property $pending 'status') -ne 'runtime-pending' -or
            (Require-Property $pending 'disposition') -ne 'runtime-pending') {
            Fail 'INVALID_FIELD_VALUE'
        }
        if ((Require-Property $candidate 'status') -ne 'retained' -or
            (Require-Property $candidate 'disposition') -ne 'production') {
            Fail 'RETENTION_INCOMPLETE'
        }
        foreach ($axis in @('execution', 'contract', 'effect')) {
            if ((Require-Property $candidate "${axis}_verdict") -ne 'pass') {
                Fail 'FAILED_RUNTIME_VERDICT'
            }
        }
        $expectedEffect = Require-Property $candidate 'expected_effect_id'
        $actualEffect = Require-Property $candidate 'actual_effect_id'
        Require-Id $expectedEffect
        Require-Id $actualEffect
        if ($expectedEffect -ne (Require-Property $pending 'expected_effect_id')) {
            Fail 'IDENTITY_CHAIN_MISMATCH'
        }
        if ($actualEffect -ne $expectedEffect) {
            Fail 'UNEXPECTED_EFFECT'
        }
        foreach ($idArrayName in @('evidence_requirement_ids', 'stop_condition_ids')) {
            $pendingIds = Require-Array (Require-Property $pending $idArrayName) 8
            $candidateIds = Require-Array (Require-Property $candidate $idArrayName) 8
            if ($pendingIds.Count -lt 1 -or ($pendingIds -join '|') -ne ($candidateIds -join '|')) {
                Fail 'IDENTITY_CHAIN_MISMATCH'
            }
            foreach ($logicalId in $candidateIds) {
                Require-Id $logicalId
            }
        }
        $runId = Require-Property $candidate 'run_id'
        Require-Id $runId
        $pendingDigests = Require-Property $pending 'identity_digests'
        $candidateDigests = Require-Property $candidate 'identity_digests'
        Require-ExactProperties $pendingDigests $staticDigestNames
        Require-ExactProperties $candidateDigests (@($staticDigestNames) + @('evidence_digest', 'run_digest', 'locator_digest'))
        foreach ($digestName in $staticDigestNames) {
            $expected = Require-Property $pendingDigests $digestName
            $actual = Require-Property $candidateDigests $digestName
            Require-Digest $expected
            Require-Digest $actual
            if ($pendingBoundDigestNames -contains $digestName -and $actual -ne $expected) {
                Fail 'IDENTITY_CHAIN_MISMATCH'
            }
        }
        if ($candidateDigests.dossier_digest -ne $ExpectedDigests.dossier_digest -or
            $candidateDigests.registry_digest -ne $ExpectedDigests.registry_digest -or
            $candidateDigests.coverage_digest -ne $ExpectedDigests.coverage_digest -or
            $candidateDigests.matrix_digest -ne $ExpectedDigests.matrix_digest -or
            $candidateDigests.build_profile_digest -ne $matrixCandidate.build_profile_digest) {
            Fail 'IDENTITY_CHAIN_MISMATCH'
        }
        foreach ($digestName in @('evidence_digest', 'run_digest', 'locator_digest')) {
            if ($candidateDigests.PSObject.Properties.Name -notcontains $digestName) {
                Fail 'EVIDENCE_CHAIN_INCOMPLETE'
            }
            Require-Digest $candidateDigests.$digestName
        }
        $runText = "$runId|$($candidateDigests.evidence_digest)|$($candidateDigests.build_manifest_digest)"
        $runDigest = Get-Sha256Hex ([Text.Encoding]::UTF8.GetBytes($runText))
        if ($candidateDigests.run_digest -ne $runDigest) {
            Fail 'IDENTITY_CHAIN_MISMATCH'
        }
    }
}

function Write-Result([string]$Verdict, [string[]]$ReasonCodes) {
    $result = [ordered]@{
        schema_version = 'x4-admission-result.v1'
        dossier_id = $script:dossierId
        verdict = $Verdict
        reason_codes = @($ReasonCodes | Select-Object -First 32)
        dossier_digest = $script:dossierDigest
        diagnostic_id = $script:validationContext
        overridden_finding_ids = @($script:overriddenFindingIds)
        negative_fixture_results = @($script:negativeFixtureResults)
    }
    Write-Output ($result | ConvertTo-Json -Compress -Depth 8)
}

try {
    $script:validationContext = 'dossier-read'
    $dossierRead = Read-BoundedJson $DossierPath 'x4-integration-dossier.v1' 'UNSUPPORTED_DOSSIER_SCHEMA'
    $script:dossierDigest = Get-Sha256Hex $dossierRead.Bytes
    $script:validationContext = 'registry-read'
    $registryRead = Read-BoundedJson $RegistryPath 'x4-known-failures.v1' 'UNSUPPORTED_REGISTRY_SCHEMA'
    $script:validationContext = 'registry-validation'
    $registryInfo = Test-Registry $registryRead.Value
    if (-not [string]::IsNullOrWhiteSpace($CoveragePath) -or -not [string]::IsNullOrWhiteSpace($FixturePath)) {
        if ([string]::IsNullOrWhiteSpace($CoveragePath) -or [string]::IsNullOrWhiteSpace($FixturePath)) {
            Fail 'MISSING_INPUT'
        }
        $script:validationContext = 'fixture-read'
        $fixtureRead = Read-BoundedJson $FixturePath 'x4-negative-fixtures.v1' 'UNSUPPORTED_FIXTURE_SCHEMA'
        $script:validationContext = 'fixture-validation'
        $fixtureMap = Test-FixtureBundle $fixtureRead.Value $registryInfo.Ids
        $script:validationContext = 'coverage-read'
        $coverageRead = Read-BoundedJson $CoveragePath 'x4-known-failure-coverage.v1' 'UNSUPPORTED_COVERAGE_SCHEMA'
        $script:validationContext = 'coverage-validation'
        Test-Coverage $coverageRead.Value $registryInfo $fixtureMap
    }
    $evidenceInputs = @($VerifiedLocatorPath, $PendingLedgerPath, $CandidateMatrixPath)
    $hasCompleteRuntimeInputs = @($evidenceInputs | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -eq 0
    $allowRuntimeResolution = -not $ValidateFixture -and $hasCompleteRuntimeInputs -and
        $dossierRead.Value.seam_id -ne 'validation-fixture-only'
    $script:validationContext = 'dossier-validation'
    $knownFindings = @(Test-Dossier $dossierRead.Value $registryInfo.Ids $allowRuntimeResolution)
    if ($knownFindings.Count -gt 0) {
        if ([string]::IsNullOrWhiteSpace($OverridePath)) { Fail 'KNOWN_FAILURE_BLOCKED' }
        $script:validationContext = 'owner-override-read'
        $overrideRead = Read-BoundedJson $OverridePath 'x4-owner-override.v1' 'UNSUPPORTED_OVERRIDE_SCHEMA'
        $script:validationContext = 'owner-override-authority'
        $authority = Test-ProductionExactOwnerOverride `
            -Override $overrideRead.Value `
            -ExpectedDossierId $script:dossierId `
            -ExpectedDossierDigest $script:dossierDigest `
            -KnownFindingIds @($knownFindings | ForEach-Object id)
        if (-not $authority.authority_ready) { Fail ([string]$authority.status) }
        $script:validationContext = 'owner-override-evaluator'
        $script:overriddenFindingIds = @(Test-OwnerOverride $overrideRead.Value $knownFindings)
        $knownFindings = @($knownFindings | Where-Object { $script:overriddenFindingIds -notcontains $_.id })
        if ($knownFindings.Count -gt 0) { Fail 'KNOWN_FAILURE_BLOCKED' }
    }
    elseif (-not [string]::IsNullOrWhiteSpace($OverridePath)) {
        Fail 'OVERRIDE_SCOPE_MISMATCH'
    }
    if ($ValidateFixture) {
        Write-Result 'validation-passed' @('VALIDATION_PASSED')
        exit 0
    }
    if ($dossierRead.Value.seam_id -eq 'validation-fixture-only') { Fail 'VALIDATION_FIXTURE_NOT_ADMISSIBLE' }
    $trustedInputs = [ordered]@{
        dossier = Join-Path $PSScriptRoot 'contracts/phase-05.1-dossier.v1.json'
        registry = Join-Path $PSScriptRoot 'contracts/known-failures.v1.json'
        coverage = Join-Path $PSScriptRoot 'contracts/coverage.v1.json'
        fixtures = Join-Path $PSScriptRoot 'fixtures/negative-fixtures.v1.json'
        matrix = Join-Path (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)) 'tests/x4-candidates/phase-05.1-candidates.v1.json'
        pending = Join-Path (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)) 'tests/x4-candidates/phase-05.1-candidate-ledger.v1.json'
    }
    $actualInputs = [ordered]@{
        dossier = $DossierPath; registry = $RegistryPath; coverage = $CoveragePath; fixtures = $FixturePath
        matrix = $CandidateMatrixPath; pending = $PendingLedgerPath
    }
    foreach ($name in $trustedInputs.Keys) {
        if ([string]::IsNullOrWhiteSpace($actualInputs[$name]) -or
            [IO.Path]::GetFullPath($actualInputs[$name]) -ne [IO.Path]::GetFullPath($trustedInputs[$name])) {
            Fail 'UNTRUSTED_EVIDENCE_SOURCE'
        }
    }
    if (-not [string]::IsNullOrWhiteSpace($SanitizedLedgerPath)) {
        Fail 'UNVERIFIED_LEDGER_INPUT'
    }
    if (-not [string]::IsNullOrWhiteSpace($VerifiedLocatorPath)) {
        if (-not (Test-Path -LiteralPath $retentionVerifierPath -PathType Leaf)) { Fail 'RETENTION_VERIFIER_MISSING' }
        $verifiedOutput = @(& pwsh -NoProfile -File $retentionVerifierPath -VerifyLocatorPath $VerifiedLocatorPath 2>&1)
        if ($LASTEXITCODE -ne 0) {
            $retentionJson = @($verifiedOutput | ForEach-Object { $_.ToString() } | Where-Object { $_.TrimStart().StartsWith('{') }) | Select-Object -Last 1
            if ($null -eq $retentionJson) { Fail 'RETENTION_VERIFICATION_FAILED' }
            try { $retentionResult = $retentionJson | ConvertFrom-Json -Depth 8 -DateKind String }
            catch { Fail 'RETENTION_VERIFICATION_FAILED' }
            if ($retentionResult.reason_code -isnot [string] -or $retentionResult.reason_code -notmatch '^[A-Z0-9_]{3,64}$') {
                Fail 'RETENTION_VERIFICATION_FAILED'
            }
            Fail ([string]$retentionResult.reason_code)
        }
        $verifiedJson = @($verifiedOutput | ForEach-Object { $_.ToString() } | Where-Object { $_.TrimStart().StartsWith('{') })
        if ($verifiedJson.Count -ne 1 -or [Text.Encoding]::UTF8.GetByteCount($verifiedJson[0]) -gt $maxInputBytes) {
            Fail 'RETENTION_VERIFICATION_FAILED'
        }
        try { $ledger = $verifiedJson[0] | ConvertFrom-Json -Depth 32 -DateKind String }
        catch { Fail 'RETENTION_VERIFICATION_FAILED' }
        $matrixRead = Read-BoundedJson $CandidateMatrixPath 'phase-05.1-candidates.v1' 'UNSUPPORTED_MATRIX_SCHEMA'
        $pendingLedgerRead = Read-BoundedJson $PendingLedgerPath 'phase-05.1-candidate-ledger.v1' 'UNSUPPORTED_LEDGER_SCHEMA'
        $expectedDigests = [pscustomobject]@{
            dossier_digest = $script:dossierDigest
            registry_digest = Get-Sha256Hex $registryRead.Bytes
            coverage_digest = Get-Sha256Hex $coverageRead.Bytes
            matrix_digest = Get-Sha256Hex $matrixRead.Bytes
        }
        $classification = Test-EvidenceChain $ledger $pendingLedgerRead.Value $matrixRead.Value $expectedDigests
        if ($classification -eq 'authenticated-local-contract') {
            Write-Result 'chain-verified-x4-pending' @('CHAIN_VERIFIED_X4_PENDING')
            exit 0
        }
        Fail 'OBSERVED_X4_EVIDENCE_INCOMPLETE'
    }
    Fail 'RUNTIME_ADMISSION_PENDING'
    if ($dossierRead.Value.seam_id -eq 'validation-fixture-only') { Fail 'VALIDATION_FIXTURE_NOT_ADMISSIBLE' }
    if (@($evidenceInputs | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -gt 0 -or
        [string]::IsNullOrWhiteSpace($CoveragePath) -or [string]::IsNullOrWhiteSpace($FixturePath)) {
        Fail 'MISSING_ADMISSION_EVIDENCE'
    }
    if (@($evidenceInputs | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }).Count -gt 0) {
        if (@($evidenceInputs | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -gt 0 -or
            [string]::IsNullOrWhiteSpace($CoveragePath) -or [string]::IsNullOrWhiteSpace($FixturePath)) {
            Fail 'MISSING_ADMISSION_EVIDENCE'
        }
        $script:validationContext = 'candidate-matrix-read'
        $matrixRead = Read-BoundedJson $CandidateMatrixPath 'phase-05.1-candidates.v1' 'UNSUPPORTED_MATRIX_SCHEMA'
        $script:validationContext = 'pending-ledger-read'
        $pendingLedgerRead = Read-BoundedJson $PendingLedgerPath 'phase-05.1-candidate-ledger.v1' 'UNSUPPORTED_LEDGER_SCHEMA'
        $script:validationContext = 'sanitized-ledger-read'
        $ledgerRead = Read-BoundedJson $SanitizedLedgerPath 'phase-05.1-candidate-ledger.v1' 'UNSUPPORTED_LEDGER_SCHEMA'
        $expectedDigests = [pscustomobject]@{
            dossier_digest = $script:dossierDigest
            registry_digest = Get-Sha256Hex $registryRead.Bytes
            coverage_digest = Get-Sha256Hex $coverageRead.Bytes
            matrix_digest = Get-Sha256Hex $matrixRead.Bytes
        }
        $script:validationContext = 'evidence-chain-validation'
        Test-EvidenceChain $ledgerRead.Value $pendingLedgerRead.Value $matrixRead.Value $expectedDigests
    }
    Write-Result 'admissible' @('ADMISSIBLE')
    exit 0
}
catch {
    Write-Result 'non-admissible' @($script:failureCode)
    exit 1
}
