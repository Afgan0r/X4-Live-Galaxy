[CmdletBinding()]
param(
    [ValidateSet('retention', 'handback', 'retention-admission')]
    [string]$Case = 'retention'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$toolRoot = Split-Path -Parent $PSScriptRoot
$root = Split-Path -Parent (Split-Path -Parent $toolRoot)
$retentionPath = Join-Path $toolRoot 'retain-evidence.ps1'
$sanitizedContractPath = Join-Path $toolRoot 'contracts/sanitized-ledger.v1.json'
$builderPath = Join-Path $toolRoot 'build-candidate-extension.ps1'
$dossierPath = Join-Path $toolRoot 'contracts/phase-05.1-dossier.v1.json'
$matrixPath = Join-Path $root 'tests/x4-candidates/phase-05.1-candidates.v1.json'
$pendingLedgerPath = Join-Path $root 'tests/x4-candidates/phase-05.1-candidate-ledger.v1.json'
$admissionPath = Join-Path $toolRoot 'x4-admission.ps1'
$registryPath = Join-Path $toolRoot 'contracts/known-failures.v1.json'
$coveragePath = Join-Path $toolRoot 'contracts/coverage.v1.json'
$fixturePath = Join-Path $toolRoot 'fixtures/negative-fixtures.v1.json'

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

function New-DirectoryReparsePoint([string]$Path, [string]$Target) {
    $itemType = if ($IsWindows) { 'Junction' } else { 'SymbolicLink' }
    $null = New-Item -ItemType $itemType -Path $Path -Target $Target
}

function ConvertTo-CanonicalValue($Value) {
    if ($null -eq $Value -or $Value -is [string] -or $Value -is [bool] -or
        $Value -is [byte] -or $Value -is [int16] -or $Value -is [int32] -or
        $Value -is [int64] -or $Value -is [decimal] -or $Value -is [double]) {
        return $Value
    }
    if ($Value -is [System.Collections.IDictionary]) {
        $dictionary = [ordered]@{}
        foreach ($key in @($Value.Keys | Sort-Object)) {
            $dictionary[$key] = ConvertTo-CanonicalValue $Value[$key]
        }
        return $dictionary
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

function Set-RowDigest($Row) {
    $payload = [ordered]@{}
    foreach ($property in @($Row.PSObject.Properties | Sort-Object Name)) {
        if ($property.Name -notin @('digest_algorithm', 'canonical_digest_payload', 'record_digest')) {
            $payload[$property.Name] = $property.Value
        }
    }
    $Row.canonical_digest_payload = ConvertTo-CanonicalJson $payload
    $Row.record_digest = Get-TextDigest $Row.canonical_digest_payload
}

function Write-EvidenceRows([string]$Path, $Rows) {
    foreach ($row in @($Rows)) { Set-RowDigest $row }
    Write-Utf8NoBom $Path ((@($Rows | ForEach-Object { $_ | ConvertTo-Json -Compress -Depth 32 }) -join "`n") + "`n")
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
        $null = & icacls.exe $Path /grant '*S-1-5-32-545:(R)'
        if ($LASTEXITCODE -ne 0) { throw 'Unable to broaden the retained-file ACL for the negative case.' }
    }
    else {
        [System.IO.File]::SetUnixFileMode(
            $Path,
            [System.IO.UnixFileMode]::UserRead -bor [System.IO.UnixFileMode]::UserWrite -bor
                [System.IO.UnixFileMode]::GroupRead
        )
    }
}

if ($Case -eq 'handback') {
    $ledgerPath = Join-Path $root 'tests/x4-candidates/phase-05.1-candidate-ledger.v1.json'
    $procedurePath = Join-Path $root 'tests/x4-candidates/05.1-candidate-run-procedure.md'
    foreach ($requiredPath in @($ledgerPath, $procedurePath)) {
        Assert-True (Test-Path -LiteralPath $requiredPath -PathType Leaf) "Handback artifact is missing: $requiredPath"
    }

    $ledger = Get-Content -LiteralPath $ledgerPath -Raw -Encoding utf8 | ConvertFrom-Json
    $matrix = Get-Content -LiteralPath $matrixPath -Raw -Encoding utf8 | ConvertFrom-Json
    Assert-True ($ledger.schema_version -eq 'phase-05.1-candidate-ledger.v1') 'Pending ledger schema is not versioned.'
    Assert-True ($ledger.status -eq 'runtime-pending') 'Committed ledger must remain runtime-pending.'
    Assert-True (@($ledger.candidates).Count -eq 7) 'Pending ledger must contain exactly seven rows.'
    $ledgerIds = @($ledger.candidates | ForEach-Object id | Sort-Object)
    $matrixIds = @($matrix.candidates | ForEach-Object id | Sort-Object)
    Assert-True (($ledgerIds -join '|') -eq ($matrixIds -join '|')) 'Pending ledger candidates do not match the matrix exactly.'
    foreach ($candidate in @($ledger.candidates)) {
        Assert-True ($candidate.status -eq 'runtime-pending') "Candidate $($candidate.id) is not pending."
        Assert-True ($candidate.disposition -eq 'runtime-pending') "Candidate $($candidate.id) claims a disposition."
        foreach ($axis in @('execution', 'contract', 'effect')) {
            Assert-True ($candidate."${axis}_verdict" -eq 'not_run') "Candidate $($candidate.id) claims a $axis verdict."
        }
        Assert-True (@($candidate.evidence_requirement_ids).Count -gt 0) "Candidate $($candidate.id) lacks evidence requirements."
        Assert-True (@($candidate.stop_condition_ids).Count -gt 0) "Candidate $($candidate.id) lacks stop conditions."
    }

    $procedure = Get-Content -LiteralPath $procedurePath -Raw -Encoding utf8
    foreach ($requiredPhrase in @('human-only', 'disposable', 'stop condition', 'retain-evidence.ps1', 'runtime-pending', 'scaffold-only', 'implementation gate')) {
        Assert-True ($procedure -match [regex]::Escape($requiredPhrase)) "Run procedure omits required boundary: $requiredPhrase"
    }
    Assert-True ($procedure -notmatch '(?i)[A-Z]:\\|/Users/|\\Users\\|savegame|successful X4 execution') 'Run procedure leaked a private path or claimed execution.'

    $productionDiff = & git diff --exit-code c4bc2ace2036eb2e27d2ef6a37671dfcb8b8d77e -- `
        extensions/live_galaxy/content.xml extensions/live_galaxy/ui.xml `
        extensions/live_galaxy/lua extensions/live_galaxy/md 2>&1
    Assert-True ($LASTEXITCODE -eq 0) "Production/public package changed from executor baseline: $($productionDiff -join ' | ')"
    Write-Output 'PASS: Phase 05.1 sanitized handback contract'
    exit 0
}

if ($Case -notin @('retention', 'retention-admission')) { throw "Unhandled case: $Case" }

foreach ($required in @($retentionPath, $sanitizedContractPath, $builderPath, $dossierPath)) {
    Assert-True (Test-Path -LiteralPath $required -PathType Leaf) "Required artifact is missing: $required"
}

$scratch = Join-Path ([IO.Path]::GetTempPath()) ("live-galaxy-retention-contract-{0}" -f [guid]::NewGuid().ToString('N'))
$buildRoot = Join-Path $scratch 'builds'
$reparseRoot = Join-Path $scratch 'reparse-root'
$null = New-Item -ItemType Directory -Path $scratch
try {
    $builderOutput = @(& pwsh -NoProfile -File $builderPath -BuildRoot $buildRoot -MatrixPath $matrixPath 2>&1)
    Assert-True ($LASTEXITCODE -eq 0) "Candidate builder failed: $builderOutput"
    if ($Case -eq 'retention-admission') {
        $pending = Get-Content -LiteralPath $pendingLedgerPath -Raw | ConvertFrom-Json -Depth 32
        $complete = Get-Content -LiteralPath $pendingLedgerPath -Raw | ConvertFrom-Json -Depth 32
        $complete.status = 'runtime-complete'
        $complete.evidence_classification = 'retained-runtime-evidence'
        $sanitizedRuns = @()
        foreach ($groupRoot in @(Get-ChildItem -LiteralPath $buildRoot -Directory | Sort-Object Name)) {
            $groupManifestPath = Join-Path $groupRoot.FullName 'manifest/build-manifest.v1.json'
            $groupManifest = Get-Content -LiteralPath $groupManifestPath -Raw | ConvertFrom-Json -Depth 32
            $groupManifest.execution_status = 'execution-ready'
            $groupManifest.native_execution_status = 'execution-ready-isolated'
            Write-Utf8NoBom $groupManifestPath ($groupManifest | ConvertTo-Json -Depth 32)
            $groupRunId = "p051-integration-$($groupManifest.group_id)"
            $groupEvidencePath = Join-Path $scratch "$($groupManifest.group_id).jsonl"
            Write-Utf8NoBom $groupEvidencePath (New-EvidenceStream $groupManifest $groupRunId)
            $groupRetentionRoot = Join-Path $scratch "retained-$($groupManifest.group_id)"
            $retentionOutput = @(Invoke-Retention $groupEvidencePath $groupManifestPath $groupRetentionRoot 0)
            Assert-True ($retentionOutput.Count -eq 1) 'Integrated retention emitted unexpected diagnostics.'
            $sanitizedRuns += @($retentionOutput[0] | ConvertFrom-Json -Depth 32)
        }
        Assert-True ($sanitizedRuns.Count -eq 2) 'Integrated handback did not retain exactly two build groups.'
        foreach ($sanitizedRun in $sanitizedRuns) {
            foreach ($retainedCandidate in @($sanitizedRun.candidates)) {
                $candidate = @($complete.candidates | Where-Object { $_.id -eq $retainedCandidate.candidate_id })
                Assert-True ($candidate.Count -eq 1) 'Sanitized handback contains an unknown candidate.'
                $candidate = $candidate[0]
                $candidate.status = 'retained'
                $candidate.disposition = 'production'
                $candidate.execution_verdict = $retainedCandidate.execution_verdict
                $candidate.contract_verdict = $retainedCandidate.contract_verdict
                $candidate.effect_verdict = $retainedCandidate.effect_verdict
                $candidate.actual_effect_id = $candidate.expected_effect_id
                $candidate.run_id = $sanitizedRun.run_id
                $candidate.identity_digests = $sanitizedRun.identity_digests
            }
        }
        Assert-True (@($complete.candidates | Where-Object { $_.status -eq 'retained' }).Count -eq 7) 'Sanitized handback did not cover all seven candidates.'
        $completePath = Join-Path $scratch 'completed-handback.json'
        Write-Utf8NoBom $completePath ($complete | ConvertTo-Json -Depth 32)
        $admissionOutput = @(& pwsh -NoProfile -File $admissionPath `
            -DossierPath $dossierPath -RegistryPath $registryPath -CoveragePath $coveragePath `
            -FixturePath $fixturePath -SanitizedLedgerPath $completePath `
            -PendingLedgerPath $pendingLedgerPath -CandidateMatrixPath $matrixPath 2>&1)
        Assert-True ($LASTEXITCODE -eq 0) "Production admission rejected retained handback: $($admissionOutput -join ' | ')"
        $admissionResult = @($admissionOutput | Where-Object { $_ -isnot [Management.Automation.ErrorRecord] })[-1] | ConvertFrom-Json
        Assert-True ($admissionResult.verdict -eq 'admissible') 'Integrated retained handback did not become admissible.'
        Write-Output 'PASS: retention-to-admission integration contract'
        exit 0
    }
    $groupRoot = Join-Path $buildRoot 'p051-build-read-only-shared'
    $manifestPath = Join-Path $groupRoot 'manifest/build-manifest.v1.json'
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    $scaffoldEvidencePath = Join-Path $scratch 'scaffold-evidence.jsonl'
    Write-Utf8NoBom $scaffoldEvidencePath (New-EvidenceStream $manifest 'p051-scaffold-rejected')
    Assert-Rejected $scaffoldEvidencePath $manifestPath (Join-Path $scratch 'reject-scaffold') 'scaffold-only manifest'
    $manifest.execution_status = 'execution-ready'
    Write-Utf8NoBom $manifestPath ($manifest | ConvertTo-Json -Depth 32)
    Assert-Rejected $scaffoldEvidencePath $manifestPath (Join-Path $scratch 'reject-native-pending') 'unproven blocking native boundary'
    $manifest.native_execution_status = 'execution-ready-isolated'
    Write-Utf8NoBom $manifestPath ($manifest | ConvertTo-Json -Depth 32)
    $runId = 'p051-retention-contract-run'
    $evidencePath = Join-Path $scratch 'runtime-evidence.jsonl'
    $validStream = New-EvidenceStream $manifest $runId
    Write-Utf8NoBom $evidencePath $validStream

    $reparseTarget = Join-Path $scratch 'reparse-target'
    $null = New-Item -ItemType Directory -Path $reparseTarget
    New-DirectoryReparsePoint $reparseRoot $reparseTarget
    $reparseOutput = Invoke-Retention $evidencePath $manifestPath $reparseRoot 1
    Assert-True (($reparseOutput -join "`n") -match '"reason_code":"DESTINATION_REPARSE_POINT_REJECTED"') 'Retention did not reject the destination reparse point.'
    Assert-True (@(Get-ChildItem -LiteralPath $reparseTarget -Force).Count -eq 0) 'Retention wrote through a destination reparse point.'

    $retentionRoot = Join-Path $scratch 'retained'
    $output = @(Invoke-Retention $evidencePath $manifestPath $retentionRoot 0)
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
    $verified = @(Invoke-Verification $locatorPath 0)
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

    $executionRewrite = @($validStream.Trim().Split("`n") | ForEach-Object { $_ | ConvertFrom-Json })
    $executionRewrite[0].execution_verdict = 'fail'
    $executionRewrite[0].failure_point = 'execution'
    $executionRewrite[0].failure_reason = 'execution_exception'
    $executionRewritePath = Join-Path $scratch 'execution-rewrite.jsonl'
    Write-EvidenceRows $executionRewritePath $executionRewrite
    Assert-Rejected $executionRewritePath $manifestPath (Join-Path $scratch 'reject-execution-rewrite') 'execution failure rewritten to pass'

    $contractRewrite = @($validStream.Trim().Split("`n") | ForEach-Object { $_ | ConvertFrom-Json })
    $contractRewrite[1].contract_verdict = 'fail'
    $contractRewrite[1].failure_point = 'contract'
    $contractRewrite[1].failure_reason = 'contract_rejected'
    $contractRewritePath = Join-Path $scratch 'contract-rewrite.jsonl'
    Write-EvidenceRows $contractRewritePath $contractRewrite
    Assert-Rejected $contractRewritePath $manifestPath (Join-Path $scratch 'reject-contract-rewrite') 'contract failure rewritten to pass'

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
    if (Test-Path -LiteralPath $reparseRoot) { [IO.Directory]::Delete($reparseRoot) }
    if (Test-Path -LiteralPath $scratch) {
        Remove-Item -LiteralPath $scratch -Recurse -Force
    }
}
