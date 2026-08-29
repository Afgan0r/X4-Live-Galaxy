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
    [string]$OverridePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

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
        Require-Id (Require-Property $fixture 'finding_id')
        Require-Bool (Require-Property $fixture 'enabled')
        if (-not $fixture.enabled) {
            Fail 'SKIPPED_NEGATIVE_FIXTURE'
        }
        if ((Require-Property $fixture 'expected_reason_code') -ne 'KNOWN_FAILURE_BLOCKED') {
            Fail 'PASSING_NEGATIVE_FIXTURE'
        }
        $map[$id] = $fixture
    }
    return $map
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

function Test-Dossier($Dossier, [string[]]$FailureClassIds) {
    $script:dossierId = Require-Property $Dossier 'dossier_id'
    Require-Id $script:dossierId
    Require-Id (Require-Property $Dossier 'seam_id')
    $evidenceMap = Get-EvidenceMap $Dossier
    $dimensions = Require-Array (Require-Property $Dossier 'dimensions') $maxDimensions
    Assert-UniqueIds $dimensions
    $dimensionIds = @()
    foreach ($dimension in $dimensions) {
        $id = Require-Property $dimension 'id'
        Require-Id $id
        $status = Require-Property $dimension 'status'
        if ($status -notin @('EVIDENCED', 'CONFLICTING', 'UNKNOWN')) {
            Fail 'INVALID_EVIDENCE_STATUS'
        }
        Assert-Provenance $dimension $evidenceMap
        if ($status -ne 'EVIDENCED') {
            Fail 'UNRESOLVED_EVIDENCE'
        }
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
    foreach ($finding in $findings) {
        Require-Id (Require-Property $finding 'id')
        $failureClassId = Require-Property $finding 'failure_class_id'
        if ($FailureClassIds -notcontains $failureClassId) {
            Fail 'INVALID_EVIDENCE_REFERENCE'
        }
        $disposition = Require-Property $finding 'disposition'
        if ($disposition -notin @('resolved', 'known-failure')) {
            Fail 'INVALID_FIELD_VALUE'
        }
        if ($disposition -eq 'known-failure') {
            Fail 'KNOWN_FAILURE_BLOCKED'
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
    $script:validationContext = 'dossier-validation'
    Test-Dossier $dossierRead.Value $registryInfo.Ids
    Write-Result 'admissible' @('ADMISSIBLE')
    exit 0
}
catch {
    Write-Result 'non-admissible' @($script:failureCode)
    exit 1
}
