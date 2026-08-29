[CmdletBinding()]
param(
    [ValidateSet('dossier')]
    [string]$Case = 'dossier'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$toolRoot = Split-Path -Parent $PSScriptRoot
$contractsRoot = Join-Path $toolRoot 'contracts'
$admissionPath = Join-Path $toolRoot 'x4-admission.ps1'
$dossierPath = Join-Path $contractsRoot 'dossier.v1.json'
$registryPath = Join-Path $contractsRoot 'known-failures.v1.json'

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

function Invoke-Admission($Dossier, $Registry) {
    $tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("live-galaxy-admission-{0}" -f [guid]::NewGuid().ToString('N'))
    [void](New-Item -ItemType Directory -Path $tempRoot)
    try {
        $tempDossier = Join-Path $tempRoot 'dossier.json'
        $tempRegistry = Join-Path $tempRoot 'registry.json'
        $Dossier | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $tempDossier -Encoding utf8NoBOM
        $Registry | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $tempRegistry -Encoding utf8NoBOM
        $output = & pwsh -NoProfile -File $admissionPath -DossierPath $tempDossier -RegistryPath $tempRegistry 2>&1
        $exitCode = $LASTEXITCODE
        $jsonLine = @($output | ForEach-Object { $_.ToString() } | Where-Object { $_.TrimStart().StartsWith('{') }) | Select-Object -Last 1
        Assert-True ($null -ne $jsonLine) "Admission emitted no JSON result: $($output -join ' | ')"
        return [pscustomobject]@{
            ExitCode = $exitCode
            Result = $jsonLine | ConvertFrom-Json
            Output = @($output | ForEach-Object { $_.ToString() })
        }
    }
    finally {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }
}

function Assert-Rejected($Run, [string]$ReasonCode, [string]$Label) {
    Assert-True ($Run.ExitCode -ne 0) "$Label unexpectedly succeeded."
    Assert-True ($Run.Result.verdict -eq 'non-admissible') "$Label returned an unstable verdict."
    Assert-True (@($Run.Result.reason_codes) -contains $ReasonCode) "$Label did not report $ReasonCode."
    $joined = $Run.Output -join "`n"
    Assert-True ($joined -notmatch '(?i)[A-Z]:\\|/Users/|\\Users\\|private|secret') "$Label leaked a private path or value."
}

foreach ($path in @($admissionPath, $dossierPath, $registryPath)) {
    Assert-True (Test-Path -LiteralPath $path -PathType Leaf) "Required admission artifact is missing: $path"
}

$baseDossier = Get-Content -LiteralPath $dossierPath -Raw -Encoding utf8 | ConvertFrom-Json
$baseRegistry = Get-Content -LiteralPath $registryPath -Raw -Encoding utf8 | ConvertFrom-Json

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
