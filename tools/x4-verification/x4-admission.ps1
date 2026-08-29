[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string]$DossierPath,
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string]$RegistryPath,
    [string]$CoveragePath,
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
    if ((@($ids | Sort-Object) -join '|') -ne (@($requiredFailureClasses | Sort-Object) -join '|')) {
        Fail 'INVALID_FAILURE_REGISTRY'
    }
    return @($ids)
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
    }
    Write-Output ($result | ConvertTo-Json -Compress -Depth 8)
}

try {
    $dossierRead = Read-BoundedJson $DossierPath 'x4-integration-dossier.v1' 'UNSUPPORTED_DOSSIER_SCHEMA'
    $script:dossierDigest = Get-Sha256Hex $dossierRead.Bytes
    $registryRead = Read-BoundedJson $RegistryPath 'x4-known-failures.v1' 'UNSUPPORTED_REGISTRY_SCHEMA'
    $failureClasses = Test-Registry $registryRead.Value
    Test-Dossier $dossierRead.Value $failureClasses
    Write-Result 'admissible' @('ADMISSIBLE')
    exit 0
}
catch {
    Write-Result 'non-admissible' @($script:failureCode)
    exit 1
}
