[CmdletBinding()]
param(
    [ValidateSet('retention')]
    [string]$Case = 'retention'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$toolRoot = Split-Path -Parent $PSScriptRoot
$root = Split-Path -Parent (Split-Path -Parent $toolRoot)
$retentionPath = Join-Path $toolRoot 'retain-evidence.ps1'
$sanitizedContractPath = Join-Path $toolRoot 'contracts/sanitized-ledger.v1.json'
$builderPath = Join-Path $toolRoot 'build-candidate-extension.ps1'
$dossierPath = Join-Path $toolRoot 'contracts/dossier.v1.json'

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Write-Utf8NoBom([string]$Path, [string]$Content) {
    [System.IO.File]::WriteAllText($Path, $Content, [System.Text.UTF8Encoding]::new($false))
}

function Get-Digest([string]$Path) {
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-TextDigest([string]$Value) {
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($Value)
    return ([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes))).ToLowerInvariant()
}

function ConvertTo-CanonicalValue($Value) {
    if ($null -eq $Value -or $Value -is [string] -or $Value -is [bool] -or
        $Value -is [byte] -or $Value -is [int16] -or $Value -is [int32] -or
        $Value -is [int64] -or $Value -is [decimal] -or $Value -is [double]) {
        return $Value
    }
    if ($Value -is [System.Collections.IEnumerable] -and $Value -isnot [pscustomobject]) {
        return @($Value | ForEach-Object { ConvertTo-CanonicalValue $_ })
    }
    $result = [ordered]@{}
    foreach ($property in @($Value.PSObject.Properties | Sort-Object Name)) {
        $result[$property.Name] = ConvertTo-CanonicalValue $property.Value
    }
    return $result
}

function ConvertTo-CanonicalJson($Value) {
    return (ConvertTo-CanonicalValue $Value | ConvertTo-Json -Compress -Depth 32)
}

function New-EvidenceStream($Manifest, [string]$RunId) {
    $dossier = Get-Content -LiteralPath $dossierPath -Raw | ConvertFrom-Json
    $rows = [System.Collections.Generic.List[string]]::new()
    foreach ($candidateId in @($Manifest.candidate_ids | Sort-Object)) {
        foreach ($stage in @('execution', 'contract', 'effect')) {
            $contractVerdict = if ($stage -eq 'execution') { 'not_run' } else { 'pass' }
            $effectVerdict = if ($stage -eq 'effect') { 'pass' } else { 'not_run' }
            $payload = [ordered]@{
                actual_result = 'expected-result'
                build_id = $Manifest.build_id
                build_profile_digest = $Manifest.build_profile_digest
                candidate_id = $candidateId
                candidate_source = 'phase-05.2-research-candidate-matrix'
                completeness = 'complete'
                contract_verdict = $contractVerdict
                effect_verdict = $effectVerdict
                elapsed_game_ms = 2
                elapsed_real_ms = 1
                evidence_classification = 'local-contract-only'
                execution_verdict = 'pass'
                expected_result = 'expected-result'
                failure_point = 'none'
                failure_reason = 'none'
                game_version = 'x4-9.00-test-fixture'
                mod_list = @('live-galaxy-test')
                observation_count = 1
                prior_dossier_digest = $Manifest.dossier_digest
                prior_dossier_id = $dossier.dossier_id
                run_id = $RunId
                scenario_id = 'creative-custom-disposable-test'
                schema_version = 'runtime-evidence.v1'
                seta_state = 'not_applicable'
                stage_id = $stage
                work_units = 1
            }
            $canonical = ConvertTo-CanonicalJson $payload
            $row = [ordered]@{}
            foreach ($property in $payload.GetEnumerator()) { $row[$property.Key] = $property.Value }
            $row.digest_algorithm = 'sha256'
            $row.canonical_digest_payload = $canonical
            $row.record_digest = Get-TextDigest $canonical
            $rows.Add(($row | ConvertTo-Json -Compress -Depth 32))
        }
    }
    return (($rows -join "`n") + "`n")
}

function Invoke-Retention(
    [string]$EvidencePath,
    [string]$ManifestPath,
    [string]$DestinationRoot,
    [int]$ExpectedExitCode
) {
    $output = @(& pwsh -NoProfile -File $retentionPath -EvidencePath $EvidencePath `
        -BuildManifestPath $ManifestPath -DestinationRoot $DestinationRoot 2>&1)
    Assert-True ($LASTEXITCODE -eq $ExpectedExitCode) "Retention exit code was $LASTEXITCODE, expected $ExpectedExitCode. Output: $output"
    return @($output | ForEach-Object { $_.ToString() })
}

function Invoke-Verification([string]$LocatorPath, [int]$ExpectedExitCode) {
    $output = @(& pwsh -NoProfile -File $retentionPath -VerifyLocatorPath $LocatorPath 2>&1)
    Assert-True ($LASTEXITCODE -eq $ExpectedExitCode) "Verification exit code was $LASTEXITCODE, expected $ExpectedExitCode. Output: $output"
    return @($output | ForEach-Object { $_.ToString() })
}

function Assert-Rejected([string]$EvidencePath, [string]$ManifestPath, [string]$DestinationRoot, [string]$Label) {
    $output = Invoke-Retention $EvidencePath $ManifestPath $DestinationRoot 1
    Assert-True (($output -join "`n") -match '"verdict":"rejected"') "$Label did not return a structured rejection."
    Assert-True (-not (Test-Path -LiteralPath $DestinationRoot)) "$Label left a partial destination root."
}

function Add-BroadReadPermission([string]$Path) {
    if ($IsWindows) {
        $acl = Get-Acl -LiteralPath $Path
        $sid = [Security.Principal.SecurityIdentifier]::new('S-1-5-32-545')
        $rule = [Security.AccessControl.FileSystemAccessRule]::new(
            $sid,
            [Security.AccessControl.FileSystemRights]::Read,
            [Security.AccessControl.AccessControlType]::Allow
        )
        $acl.AddAccessRule($rule)
        Set-Acl -LiteralPath $Path -AclObject $acl
    }
    else {
        [System.IO.File]::SetUnixFileMode(
            $Path,
            [System.IO.UnixFileMode]::UserRead -bor [System.IO.UnixFileMode]::UserWrite -bor
                [System.IO.UnixFileMode]::GroupRead
        )
    }
}

if ($Case -ne 'retention') { throw "Unhandled case: $Case" }

foreach ($required in @($retentionPath, $sanitizedContractPath, $builderPath, $dossierPath)) {
    Assert-True (Test-Path -LiteralPath $required -PathType Leaf) "Required artifact is missing: $required"
}

$scratch = Join-Path ([IO.Path]::GetTempPath()) ("live-galaxy-retention-contract-{0}" -f [guid]::NewGuid().ToString('N'))
$buildRoot = Join-Path $scratch 'builds'
$null = New-Item -ItemType Directory -Path $scratch
try {
    $builderOutput = @(& pwsh -NoProfile -File $builderPath -BuildRoot $buildRoot 2>&1)
    Assert-True ($LASTEXITCODE -eq 0) "Candidate builder failed: $builderOutput"
    $groupRoot = Join-Path $buildRoot 'p051-build-read-only-shared'
    $manifestPath = Join-Path $groupRoot 'manifest/build-manifest.v1.json'
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    $runId = 'p051-retention-contract-run'
    $evidencePath = Join-Path $scratch 'runtime-evidence.jsonl'
    $validStream = New-EvidenceStream $manifest $runId
    Write-Utf8NoBom $evidencePath $validStream

    $retentionRoot = Join-Path $scratch 'retained'
    $output = Invoke-Retention $evidencePath $manifestPath $retentionRoot 0
    Assert-True ($output.Count -eq 1) 'Successful retention emitted diagnostics besides the sanitized object.'
    $sanitized = $output[0] | ConvertFrom-Json
    $contract = Get-Content -LiteralPath $sanitizedContractPath -Raw | ConvertFrom-Json
    Assert-True ($sanitized.schema_version -eq $contract.schema_version) 'Sanitized schema version mismatch.'
    Assert-True ($sanitized.run_id -eq $runId) 'Sanitized run identity mismatch.'
    Assert-True ($sanitized.retention_disposition -eq 'retained') 'Retention disposition mismatch.'
    Assert-True (@($sanitized.candidates).Count -eq @($manifest.candidate_ids).Count) 'Sanitized candidate count mismatch.'
    foreach ($candidate in @($sanitized.candidates)) {
        Assert-True ($candidate.execution_verdict -eq 'pass') 'Execution verdict was not retained.'
        Assert-True ($candidate.contract_verdict -eq 'pass') 'Contract verdict was not retained.'
        Assert-True ($candidate.effect_verdict -eq 'pass') 'Effect verdict was not retained.'
        Assert-True ($candidate.disposition -eq 'retain') 'Candidate disposition mismatch.'
    }
    foreach ($digestName in @($contract.required_identity_digests)) {
        Assert-True ([string]$sanitized.identity_digests.$digestName -match '^[a-f0-9]{64}$') "Missing sanitized digest: $digestName"
    }
    $serialized = $sanitized | ConvertTo-Json -Compress -Depth 32
    foreach ($privateValue in @($scratch, 'x4-9.00-test-fixture', 'live-galaxy-test', 'creative-custom-disposable-test', 'expected-result', 'phase-05.2-research-candidate-matrix')) {
        Assert-True ($serialized -notlike "*$privateValue*") "Sanitized output leaked private/raw value: $privateValue"
    }

    $runRoot = Join-Path $retentionRoot $runId
    $locatorPath = Join-Path $runRoot 'locator.v1.json'
    $retainedEvidencePath = Join-Path $runRoot 'runtime-evidence.v1.jsonl'
    $retainedManifestPath = Join-Path $runRoot 'build-manifest.v1.json'
    foreach ($retained in @($locatorPath, $retainedEvidencePath, $retainedManifestPath)) {
        Assert-True (Test-Path -LiteralPath $retained -PathType Leaf) "Retained artifact is missing: $retained"
    }
    Assert-True ((Get-Digest $retainedEvidencePath) -eq (Get-Digest $evidencePath)) 'Retained evidence digest changed.'
    Assert-True ((Get-Digest $retainedManifestPath) -eq (Get-Digest $manifestPath)) 'Retained build manifest digest changed.'
    $verified = Invoke-Verification $locatorPath 0
    Assert-True ($verified.Count -eq 1) 'Verification emitted diagnostics besides the sanitized object.'
    Assert-True (($verified[0] | ConvertFrom-Json).identity_digests.locator_digest -eq $sanitized.identity_digests.locator_digest) 'Locator reread digest changed.'

    $partialPath = Join-Path $scratch 'partial.jsonl'
    Write-Utf8NoBom $partialPath $validStream.TrimEnd("`r", "`n")
    Assert-Rejected $partialPath $manifestPath (Join-Path $scratch 'reject-partial') 'partial line'

    $unknownRows = @($validStream.Trim().Split("`n") | ForEach-Object { $_ | ConvertFrom-Json })
    $unknownRows[0].schema_version = 'runtime-evidence.v999'
    Write-Utf8NoBom (Join-Path $scratch 'unknown.jsonl') ((@($unknownRows | ForEach-Object { $_ | ConvertTo-Json -Compress -Depth 32 }) -join "`n") + "`n")
    Assert-Rejected (Join-Path $scratch 'unknown.jsonl') $manifestPath (Join-Path $scratch 'reject-schema') 'unknown schema'

    $digestRows = @($validStream.Trim().Split("`n") | ForEach-Object { $_ | ConvertFrom-Json })
    $digestRows[0].record_digest = '0' * 64
    Write-Utf8NoBom (Join-Path $scratch 'digest.jsonl') ((@($digestRows | ForEach-Object { $_ | ConvertTo-Json -Compress -Depth 32 }) -join "`n") + "`n")
    Assert-Rejected (Join-Path $scratch 'digest.jsonl') $manifestPath (Join-Path $scratch 'reject-digest') 'digest mismatch'

    $identityRows = @($validStream.Trim().Split("`n") | ForEach-Object { $_ | ConvertFrom-Json })
    $identityRows[0].build_profile_digest = '1' * 64
    Write-Utf8NoBom (Join-Path $scratch 'identity.jsonl') ((@($identityRows | ForEach-Object { $_ | ConvertTo-Json -Compress -Depth 32 }) -join "`n") + "`n")
    Assert-Rejected (Join-Path $scratch 'identity.jsonl') $manifestPath (Join-Path $scratch 'reject-identity') 'identity mismatch'

    $oversizedPath = Join-Path $scratch 'oversized.jsonl'
    Write-Utf8NoBom $oversizedPath (($validStream.TrimEnd() + "`n") * 9)
    Assert-Rejected $oversizedPath $manifestPath (Join-Path $scratch 'reject-size') 'excess rows or bytes'

    Assert-Rejected (Join-Path $scratch 'missing.jsonl') $manifestPath (Join-Path $scratch 'reject-missing') 'missing evidence'

    Add-BroadReadPermission $retainedEvidencePath
    $permissionOutput = Invoke-Verification $locatorPath 1
    Assert-True (($permissionOutput -join "`n") -match '"verdict":"rejected"') 'Permission mismatch did not block sanitized verification output.'

    Write-Output 'PASS: evidence retention contract'
}
finally {
    if (Test-Path -LiteralPath $scratch) {
        Remove-Item -LiteralPath $scratch -Recurse -Force
    }
}
