[CmdletBinding()]
param(
    [ValidateSet('retention', 'retention-platform', 'handback', 'retention-admission', 'preallocation-bounds')]
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
$conformancePath = Join-Path $toolRoot 'x4-package-conformance.ps1'
$registryPath = Join-Path $toolRoot 'contracts/known-failures.v1.json'
$coveragePath = Join-Path $toolRoot 'contracts/coverage.v1.json'
$fixturePath = Join-Path $toolRoot 'fixtures/negative-fixtures.v1.json'
$producerModulePath = Join-Path $toolRoot 'producer-attestation.psm1'

Import-Module $producerModulePath -Force

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

function New-SparseFile([string]$Path, [long]$Length) {
    $stream = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try { $stream.SetLength($Length) }
    finally { $stream.Dispose() }
}

function Assert-PreallocationGate([string]$Path, [string]$FunctionName) {
    $source = Get-Content -LiteralPath $Path -Raw
    $start = $source.IndexOf("function $FunctionName", [StringComparison]::Ordinal)
    Assert-True ($start -ge 0) "$FunctionName is missing from $Path."
    $next = $source.IndexOf("`nfunction ", $start + 1, [StringComparison]::Ordinal)
    if ($next -lt 0) { $next = $source.Length }
    $body = $source.Substring($start, $next - $start)
    $lengthGate = $body.IndexOf('.Length -gt', [StringComparison]::Ordinal)
    $allocation = $body.IndexOf('ReadAllBytes', [StringComparison]::Ordinal)
    Assert-True ($lengthGate -ge 0 -and $allocation -ge 0 -and $lengthGate -lt $allocation) `
        "$FunctionName allocates before enforcing its metadata length bound."
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

function Get-CertificateBytes($Certificate) {
    $fields = @(
        'schema_version', 'certificate_id', 'root_id', 'root_spki_sha256',
        'delegated_spki_sha256', 'windows_key_name', 'purpose', 'epoch',
        'scope', 'algorithm', 'not_before', 'not_after', 'policy_digest'
    )
    $builder = [Text.StringBuilder]::new()
    foreach ($field in $fields) {
        $value = [string]$Certificate.$field
        [void]$builder.Append($field).Append('=').Append($value.Length).Append(':').Append($value).Append("`n")
    }
    return [Text.Encoding]::UTF8.GetBytes($builder.ToString())
}

function New-TestCertificate($Root, [string]$RootDigest, $Delegated, [string]$Purpose, [string]$Scope) {
    [byte[]]$delegatedSpki = $Delegated.ExportSubjectPublicKeyInfo()
    $certificate = [ordered]@{
        schema_version = 'x4-delegated-purpose-certificate.v1'
        certificate_id = "TEST-ONLY-$Purpose"
        root_id = 'live-galaxy-owner-root-v1'
        root_spki_sha256 = $RootDigest
        delegated_spki_sha256 = ([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($delegatedSpki))).ToLowerInvariant()
        delegated_spki_der_base64 = [Convert]::ToBase64String($delegatedSpki)
        windows_key_name = "TEST-ONLY-$Purpose"
        purpose = $Purpose
        epoch = 1
        scope = $Scope
        algorithm = 'ECDSA_P256_SHA256'
        not_before = '2025-01-01T00:00:00Z'
        not_after = '2035-01-01T00:00:00Z'
        policy_digest = Get-TextDigest "$Purpose|1|$Scope"
    }
    [byte[]]$signature = $Root.SignData(
        (Get-CertificateBytes ([pscustomobject]$certificate)),
        [Security.Cryptography.HashAlgorithmName]::SHA256,
        [Security.Cryptography.DSASignatureFormat]::IeeeP1363FixedFieldConcatenation
    )
    $certificate.root_signature_base64 = [Convert]::ToBase64String($signature)
    return [pscustomobject]$certificate
}

function New-TestRetentionAuthority([string]$AuthorityPath, [string]$AnchorPath) {
    $curve = [Security.Cryptography.ECCurve]::CreateFromFriendlyName('nistP256')
    $rootSigner = [Security.Cryptography.ECDsa]::Create($curve)
    $producerSigner = [Security.Cryptography.ECDsa]::Create($curve)
    $locatorSigner = [Security.Cryptography.ECDsa]::Create($curve)
    [byte[]]$rootSpki = $rootSigner.ExportSubjectPublicKeyInfo()
    $rootDigest = ([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($rootSpki))).ToLowerInvariant()
    $producerCertificate = New-TestCertificate $rootSigner $rootDigest $producerSigner 'candidate-producer' 'candidate-evidence:exact-build'
    $locatorCertificate = New-TestCertificate $rootSigner $rootDigest $locatorSigner 'retention-locator' 'retained-evidence:exact-run'
    $anchor = [ordered]@{
        schema_version = 'x4-owner-root-anchor.v1'; status = 'configured'
        root_id = 'live-galaxy-owner-root-v1'; root_spki_der_base64 = [Convert]::ToBase64String($rootSpki)
        root_spki_sha256 = $rootDigest; algorithm = 'ECDSA_P256_SHA256'
        policy_digest = '6497cd18a4ee0286f0d566978c9225eaa168ba5d71f8fd7042a78507e1462854'
        accepted_epochs = [ordered]@{ 'owner-override' = 1; 'candidate-producer' = 1; 'retention-locator' = 1 }
        scopes = [ordered]@{
            'owner-override' = 'known-failure:exact-finding'
            'candidate-producer' = 'candidate-evidence:exact-build'
            'retention-locator' = 'retained-evidence:exact-run'
        }
    }
    $authority = [ordered]@{
        schema_version = 'retention-test-authority.v1'; marker = 'TEST-ONLY-NEVER-PRODUCTION'
        producer_certificate = $producerCertificate
        producer_private_pkcs8_base64 = [Convert]::ToBase64String($producerSigner.ExportPkcs8PrivateKey())
        locator_certificate = $locatorCertificate
        locator_private_pkcs8_base64 = [Convert]::ToBase64String($locatorSigner.ExportPkcs8PrivateKey())
    }
    Write-Utf8NoBom $AnchorPath ($anchor | ConvertTo-Json -Depth 16)
    Write-Utf8NoBom $AuthorityPath ($authority | ConvertTo-Json -Depth 16)
    return [pscustomobject]@{
        RootDigest = $rootDigest; ProducerCertificate = $producerCertificate
        ProducerSigner = $producerSigner; RootSigner = $rootSigner; LocatorSigner = $locatorSigner
    }
}

function Write-ResignedProducerMutation([string]$Path, [byte[]]$Baseline, $Authority, [scriptblock]$Apply) {
    $envelope = [Text.Encoding]::UTF8.GetString($Baseline) | ConvertFrom-Json -Depth 32 -DateKind String
    & $Apply $envelope
    $payloadJson = producer-attestation\ConvertTo-CanonicalJson $envelope.payload
    [byte[]]$payloadBytes = [Text.UTF8Encoding]::new($false).GetBytes($payloadJson)
    $envelope.payload_digest = ([Convert]::ToHexString(
        [Security.Cryptography.SHA256]::HashData($payloadBytes)
    )).ToLowerInvariant()
    $envelope.signature_base64 = [Convert]::ToBase64String(
        $Authority.ProducerSigner.SignData(
            $payloadBytes, [Security.Cryptography.HashAlgorithmName]::SHA256,
            [Security.Cryptography.DSASignatureFormat]::IeeeP1363FixedFieldConcatenation
        )
    )
    Write-Utf8NoBom $Path (producer-attestation\ConvertTo-CanonicalJson $envelope)
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

if ($Case -eq 'preallocation-bounds') {
    $scratch = Join-Path ([IO.Path]::GetTempPath()) ("live-galaxy-preallocation-{0}" -f [guid]::NewGuid().ToString('N'))
    $null = New-Item -ItemType Directory -Path $scratch
    try {
        $admissionSparse = Join-Path $scratch 'admission-over.json'
        New-SparseFile $admissionSparse 65537
        $admissionOutput = @(& pwsh -NoProfile -File $admissionPath `
            -DossierPath $admissionSparse -RegistryPath $registryPath 2>&1)
        Assert-True ($LASTEXITCODE -ne 0 -and ($admissionOutput -join "`n") -match 'BOUND_EXCEEDED') `
            'Admission did not reject max-input-plus-one with BOUND_EXCEEDED.'

        $conformanceSparse = Join-Path $scratch 'conformance-over.json'
        New-SparseFile $conformanceSparse 131073
        $conformanceOutput = @(& pwsh -NoProfile -File $conformancePath `
            -PackageRoot (Join-Path $root 'extensions/live_galaxy') `
            -ContractPath $conformanceSparse -DossierPath $dossierPath -CoveragePath $coveragePath 2>&1)
        Assert-True ($LASTEXITCODE -ne 0 -and ($conformanceOutput -join "`n") -match 'FILE_BYTES_EXCEEDED') `
            'Package conformance did not reject max-file-plus-one with FILE_BYTES_EXCEEDED.'

        $buildSparse = Join-Path $scratch 'build-over.json'
        New-SparseFile $buildSparse 262145
        $buildDestination = Join-Path $scratch 'rejected-build'
        $buildOutput = @(& pwsh -NoProfile -File $builderPath -BuildRoot $buildDestination -MatrixPath $buildSparse 2>&1)
        Assert-True ($LASTEXITCODE -ne 0 -and ($buildOutput -join "`n") -match 'INPUT_BYTES_EXCEEDED') `
            'Candidate build did not reject max-input-plus-one with INPUT_BYTES_EXCEEDED.'
        Assert-True (-not (Test-Path -LiteralPath $buildDestination)) 'Oversized build input created output.'

        $retentionSparse = Join-Path $scratch 'retention-over.json'
        New-SparseFile $retentionSparse 262145
        $retentionDestination = Join-Path $scratch 'rejected-retention'
        $retentionOutput = @(& pwsh -NoProfile -File $retentionPath `
            -EvidencePath $dossierPath -BuildManifestPath $retentionSparse `
            -DestinationRoot $retentionDestination 2>&1)
        Assert-True ($LASTEXITCODE -ne 0 -and ($retentionOutput -join "`n") -match 'BOUND_EXCEEDED') `
            'Retention did not reject max-input-plus-one with BOUND_EXCEEDED.'
        Assert-True (-not (Test-Path -LiteralPath $retentionDestination)) 'Oversized retention input created output.'

        Assert-PreallocationGate $admissionPath 'Read-BoundedJson'
        Assert-PreallocationGate $conformancePath 'Read-BoundedBytes'
        Assert-PreallocationGate $builderPath 'Read-Json'
        Assert-PreallocationGate $retentionPath 'Read-BoundedBytes'
        Write-Output 'PASS: four-reader preallocation bounds contract'
    }
    finally {
        if (Test-Path -LiteralPath $scratch) { Remove-Item -LiteralPath $scratch -Recurse -Force }
    }
    exit 0
}

if ($Case -notin @('retention', 'retention-platform', 'retention-admission')) { throw "Unhandled case: $Case" }

foreach ($required in @($retentionPath, $sanitizedContractPath, $builderPath, $dossierPath)) {
    Assert-True (Test-Path -LiteralPath $required -PathType Leaf) "Required artifact is missing: $required"
}

$scratch = Join-Path ([IO.Path]::GetTempPath()) ("live-galaxy-retention-contract-{0}" -f [guid]::NewGuid().ToString('N'))
$buildRoot = Join-Path $scratch 'builds'
$reparseRoot = Join-Path $scratch 'reparse-root'
$testHarnessPath = $null
$admissionHarnessPath = $null
$dispatcherHarnessPath = $null
$authority = $null
$null = New-Item -ItemType Directory -Path $scratch
if ($IsWindows) {
    $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    $null = & icacls.exe $scratch /inheritance:r /grant:r "*$sid`:(OI)(CI)F"
    Assert-True ($LASTEXITCODE -eq 0) 'Unable to protect retention scratch fixture.'
}
try {
    $builderOutput = @(& pwsh -NoProfile -File $builderPath -BuildRoot $buildRoot -MatrixPath $matrixPath 2>&1)
    Assert-True ($LASTEXITCODE -eq 0) "Candidate builder failed: $builderOutput"
    $pendingGroupRoot = Join-Path $buildRoot 'p051-build-read-only-shared'
    $pendingManifestPath = Join-Path $pendingGroupRoot 'manifest/build-manifest.v1.json'
    $pendingManifest = Get-Content -LiteralPath $pendingManifestPath -Raw | ConvertFrom-Json -Depth 32
    $pendingEvidencePath = Join-Path $scratch 'pending-runtime.jsonl'
    $dispatcherPath = Join-Path $root 'tools/x4-verification/run-candidate-package.ps1'
    $dispatcherOutput = @(& pwsh -NoProfile -File $dispatcherPath -GroupRoot $pendingGroupRoot -OutputPath $pendingEvidencePath 2>&1)
    Assert-True ($LASTEXITCODE -eq 0) "Candidate dispatcher failed: $($dispatcherOutput -join ' | ')"
    Assert-True (Test-Path -LiteralPath $pendingEvidencePath -PathType Leaf) 'Candidate dispatcher did not create evidence.'
    $pendingOutput = Invoke-Retention $pendingEvidencePath $pendingManifestPath (Join-Path $scratch 'reject-untrusted-runtime') 1
    Assert-True (($pendingOutput -join "`n") -match 'RETENTION_ATTESTATION_UNCONFIGURED') "Unconfigured retention authority did not fail closed: $($pendingOutput -join ' | ')"

    $tamperedManifest = Get-Content -LiteralPath $pendingManifestPath -Raw | ConvertFrom-Json -Depth 32
    $tamperedManifest.package_conformance_digest = '0' * 64
    $tamperedManifestPath = Join-Path $scratch 'tampered-package-digest.json'
    Write-Utf8NoBom $tamperedManifestPath ($tamperedManifest | ConvertTo-Json -Depth 32)
    $digestOutput = Invoke-Retention $pendingEvidencePath $tamperedManifestPath (Join-Path $scratch 'reject-package-digest') 1
    Assert-True (($digestOutput -join "`n") -match 'PACKAGE_CONFORMANCE_DIGEST_MISMATCH') 'Package conformance digest was not independently recomputed.'

    $forgedGroupRoot = Join-Path $scratch 'forged-embedded-conformance'
    Copy-Item -LiteralPath $pendingGroupRoot -Destination $forgedGroupRoot -Recurse
    $forgedManifestPath = Join-Path $forgedGroupRoot 'manifest/build-manifest.v1.json'
    $forgedManifest = Get-Content -LiteralPath $forgedManifestPath -Raw | ConvertFrom-Json -Depth 32
    $forgedEntrypointPath = Join-Path $forgedGroupRoot 'lua/live_galaxy_candidate_entry.lua'
    Add-Content -LiteralPath $forgedEntrypointPath -Value "`nrequire('live_galaxy/lua/missing_transitive')" -Encoding utf8NoBOM
    $entryRow = @($forgedManifest.generated_files | Where-Object { $_.path -eq 'lua/live_galaxy_candidate_entry.lua' })
    Assert-True ($entryRow.Count -eq 1) 'Generated entrypoint row is missing from the forgery fixture.'
    $entryRow[0].bytes = (Get-Item -LiteralPath $forgedEntrypointPath).Length
    $entryRow[0].sha256 = Get-Digest $forgedEntrypointPath
    $graphText = (Get-Digest (Join-Path $forgedGroupRoot 'content.xml')) +
        (Get-Digest (Join-Path $forgedGroupRoot 'ui.xml')) + $entryRow[0].sha256
    $forgedManifest.package_conformance.graph_digest = Get-TextDigest $graphText
    $forgedPackageText = $forgedManifest.package_conformance | ConvertTo-Json -Compress -Depth 32
    $forgedManifest.package_conformance_digest = Get-TextDigest $forgedPackageText
    Write-Utf8NoBom $forgedManifestPath ($forgedManifest | ConvertTo-Json -Depth 32)
    $forgedOutput = Invoke-Retention $pendingEvidencePath $forgedManifestPath `
        (Join-Path $scratch 'reject-embedded-forgery') 1
    Assert-True (($forgedOutput -join "`n") -match 'PACKAGE_CONFORMANCE_LIVE_MISMATCH') `
        'A self-consistent embedded conformance forgery was not rejected by a live graph rerun.'
    if ($Case -eq 'retention-platform') {
        $platformScript = Join-Path $scratch 'retain-evidence-non-windows.ps1'
        $platformSource = (Get-Content -LiteralPath $retentionPath -Raw).Replace('$script:SimulateUnsupportedPlatform = $false', '$script:SimulateUnsupportedPlatform = $true')
        Write-Utf8NoBom $platformScript $platformSource
        Copy-Item -LiteralPath (Join-Path $toolRoot 'producer-attestation.psm1') `
            -Destination (Join-Path $scratch 'producer-attestation.psm1')
        $platformRoot = Join-Path $scratch 'unsupported-platform-retained'
        $platformOutput = @(& pwsh -NoProfile -File $platformScript -EvidencePath $pendingEvidencePath `
            -BuildManifestPath $pendingManifestPath -DestinationRoot $platformRoot 2>&1)
        Assert-True ($LASTEXITCODE -ne 0) 'Simulated unsupported platform returned success.'
        Assert-True (($platformOutput -join "`n") -match 'RETENTION_ATTESTATION_PLATFORM_UNSUPPORTED') 'Unsupported platform status was not stable.'
        Assert-True (-not (Test-Path -LiteralPath $platformRoot)) 'Unsupported platform created retained output.'
        Write-Output 'PASS: evidence retention unsupported-platform contract'
        exit 0
    }
    $groupRoot = $pendingGroupRoot
    $manifestPath = $pendingManifestPath
    $manifest = $pendingManifest
    $evidencePath = $pendingEvidencePath
    $evidenceRows = @(Get-Content -LiteralPath $evidencePath | ForEach-Object { $_ | ConvertFrom-Json -Depth 16 -DateKind String })
    $runId = [string]$evidenceRows[0].run_id

    $authorityPath = Join-Path $scratch 'retention-test-authority.v1.json'
    $anchorPath = Join-Path $scratch 'test-owner-root-anchor.v1.json'
    $authority = New-TestRetentionAuthority $authorityPath $anchorPath
    $productionDispatcherPath = Join-Path $root 'tools/x4-verification/run-candidate-package.ps1'
    $dispatcherHarnessPath = Join-Path (Split-Path -Parent $productionDispatcherPath) `
        ('.run-candidate-package-test-' + [guid]::NewGuid().ToString('N') + '.ps1')
    $dispatcherHarnessSource = Get-Content -LiteralPath $productionDispatcherPath -Raw
    $dispatcherHarnessSource = $dispatcherHarnessSource.Replace(
        '$dispatcherPath = $PSCommandPath',
        "`$dispatcherPath = '$($productionDispatcherPath.Replace("'", "''"))'"
    )
    $dispatcherHarnessSource = $dispatcherHarnessSource.Replace(
        '$script:TestOnlyHarness = $false', '$script:TestOnlyHarness = $true'
    )
    $dispatcherHarnessSource = $dispatcherHarnessSource.Replace(
        "`$script:TestAuthorityPath = ''",
        "`$script:TestAuthorityPath = '$($authorityPath.Replace("'", "''"))'"
    )
    Write-Utf8NoBom $dispatcherHarnessPath $dispatcherHarnessSource
    $trustedEvidencePath = Join-Path $scratch 'trusted-runtime.jsonl'
    $trustedDispatcherOutput = @(& pwsh -NoProfile -File $dispatcherHarnessPath `
        -GroupRoot $pendingGroupRoot -OutputPath $trustedEvidencePath 2>&1)
    Assert-True ($LASTEXITCODE -eq 0) "Test-authority dispatcher failed: $($trustedDispatcherOutput -join ' | ')"
    $trustedDispatcherResult = @($trustedDispatcherOutput)[-1] | ConvertFrom-Json -Depth 16 -DateKind String
    Assert-True ($trustedDispatcherResult.retainable -eq $true -and
        $trustedDispatcherResult.attestation_status -eq 'PRODUCER_ATTESTATION_VERIFIED') `
        'Real dispatcher serialization did not produce a retainable envelope.'
    Assert-True (Test-Path -LiteralPath "$trustedEvidencePath.attestation.json" -PathType Leaf) `
        'Real dispatcher did not publish its producer envelope.'
    $evidencePath = $trustedEvidencePath
    $evidenceRows = @(Get-Content -LiteralPath $evidencePath | ForEach-Object { $_ | ConvertFrom-Json -Depth 16 -DateKind String })
    $runId = [string]$evidenceRows[0].run_id
    $testHarnessPath = Join-Path (Split-Path -Parent $retentionPath) ('.retain-evidence-test-' + [guid]::NewGuid().ToString('N') + '.ps1')
    $harnessSource = Get-Content -LiteralPath $retentionPath -Raw
    $harnessSource = $harnessSource.Replace("`$script:ProductionRootSpkiSha256 = 'UNCONFIGURED'", "`$script:ProductionRootSpkiSha256 = '$($authority.RootDigest)'")
    $harnessSource = $harnessSource.Replace('$script:TestOnlyHarness = $false', '$script:TestOnlyHarness = $true')
    $harnessSource = $harnessSource.Replace("`$script:TestAuthorityPath = ''", "`$script:TestAuthorityPath = '$($authorityPath.Replace("'", "''"))'")
    $harnessSource = $harnessSource.Replace("`$ownerRootAnchorPath = Join-Path `$PSScriptRoot 'contracts/owner-root-anchor.v1.json'", "`$ownerRootAnchorPath = '$($anchorPath.Replace("'", "''"))'")
    Write-Utf8NoBom $testHarnessPath $harnessSource

    foreach ($forbiddenDestination in @(
        (Join-Path $scratch 'steamapps/common/X4 Foundations/extensions/live_galaxy/retained'),
        (Join-Path $scratch 'staging/extensions/live_galaxy/retained'),
        (Join-Path $scratch 'Documents/Egosoft/X4/123456/save/retained')
    )) {
        $forbiddenOutput = @(& pwsh -NoProfile -File $testHarnessPath -EvidencePath $evidencePath `
            -BuildManifestPath $manifestPath -DestinationRoot $forbiddenDestination 2>&1)
        Assert-True ($LASTEXITCODE -ne 0) "Retention accepted forbidden destination: $forbiddenDestination"
        Assert-True (-not (Test-Path -LiteralPath $forbiddenDestination)) `
            "Retention created a forbidden destination: $forbiddenDestination"
    }

    $reparseTarget = Join-Path $scratch 'reparse-target'
    $null = New-Item -ItemType Directory -Path $reparseTarget
    New-DirectoryReparsePoint $reparseRoot $reparseTarget
    $reparseOutput = @(& pwsh -NoProfile -File $testHarnessPath -EvidencePath $evidencePath `
        -BuildManifestPath $manifestPath -DestinationRoot $reparseRoot 2>&1)
    Assert-True ($LASTEXITCODE -ne 0) 'Retention accepted a destination reparse point.'
    Assert-True (($reparseOutput -join "`n") -match '"reason_code":"DESTINATION_REPARSE_POINT_REJECTED"') 'Retention did not reject the destination reparse point.'
    Assert-True (@(Get-ChildItem -LiteralPath $reparseTarget -Force).Count -eq 0) 'Retention wrote through a destination reparse point.'

    $retentionRoot = Join-Path $scratch 'retained'
    $output = @(& pwsh -NoProfile -File $testHarnessPath -EvidencePath $evidencePath `
        -BuildManifestPath $manifestPath -DestinationRoot $retentionRoot 2>&1)
    Assert-True ($LASTEXITCODE -eq 0) "Test-authority retention failed: $($output -join ' | ')"
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
    $retainedProducerPath = Join-Path $runRoot 'producer-attestation.v1.json'
    foreach ($retained in @($locatorPath, $retainedEvidencePath, $retainedManifestPath, $retainedProducerPath)) {
        Assert-True (Test-Path -LiteralPath $retained -PathType Leaf) "Retained artifact is missing: $retained"
    }
    Assert-True ((Get-Digest $retainedEvidencePath) -eq (Get-Digest $evidencePath)) 'Retained evidence digest changed.'
    Assert-True ((Get-Digest $retainedManifestPath) -eq (Get-Digest $manifestPath)) 'Retained build manifest digest changed.'
    $verified = @(& pwsh -NoProfile -File $testHarnessPath -VerifyLocatorPath $locatorPath 2>&1)
    Assert-True ($LASTEXITCODE -eq 0) "Test-authority locator verification failed: $($verified -join ' | ')"
    Assert-True ($verified.Count -eq 1) 'Verification emitted diagnostics besides the sanitized object.'
    Assert-True (($verified[0] | ConvertFrom-Json).identity_digests.locator_digest -eq $sanitized.identity_digests.locator_digest) 'Locator reread digest changed.'

    [byte[]]$locatorBytes = [IO.File]::ReadAllBytes($locatorPath)
    $locatorMutations = @(
        @{ Name = 'purpose confusion'; Apply = { param($value) $value.delegation_certificate.purpose = 'candidate-producer' } },
        @{ Name = 'epoch rollback'; Apply = { param($value) $value.delegation_certificate.epoch = 0 } },
        @{ Name = 'delegated key substitution'; Apply = { param($value) $value.delegation_certificate.delegated_spki_sha256 = '0' * 64 } },
        @{ Name = 'locator signature transplant'; Apply = { param($value) $value.signature_base64 = [Convert]::ToBase64String([byte[]]::new(64)) } },
        @{ Name = 'cross-run replay'; Apply = { param($value) $value.run_id = 'replayed-run' } }
    )
    foreach ($mutation in $locatorMutations) {
        $changed = [Text.Encoding]::UTF8.GetString($locatorBytes) | ConvertFrom-Json -Depth 32 -DateKind String
        & $mutation.Apply $changed
        Write-Utf8NoBom $locatorPath ($changed | ConvertTo-Json -Depth 32)
        $mutationOutput = @(& pwsh -NoProfile -File $testHarnessPath -VerifyLocatorPath $locatorPath 2>&1)
        Assert-True ($LASTEXITCODE -ne 0) "$($mutation.Name) passed locator verification."
        [IO.File]::WriteAllBytes($locatorPath, $locatorBytes)
    }

    $producerSourcePath = "$evidencePath.attestation.json"
    [byte[]]$producerBytes = [IO.File]::ReadAllBytes($producerSourcePath)
    $producerEnvelope = [Text.Encoding]::UTF8.GetString($producerBytes) | ConvertFrom-Json -Depth 32 -DateKind String
    $producerEnvelope.payload.run_id = 'replayed-producer-run'
    Write-Utf8NoBom $producerSourcePath ($producerEnvelope | ConvertTo-Json -Depth 32)
    $producerReplay = @(& pwsh -NoProfile -File $testHarnessPath -EvidencePath $evidencePath `
        -BuildManifestPath $manifestPath -DestinationRoot (Join-Path $scratch 'reject-producer-replay') 2>&1)
    Assert-True ($LASTEXITCODE -ne 0) 'Replayed producer envelope passed retention.'
    [IO.File]::WriteAllBytes($producerSourcePath, $producerBytes)

    $baselineEnvelope = [Text.Encoding]::UTF8.GetString($producerBytes) | ConvertFrom-Json -Depth 32 -DateKind String
    $baselineCanonical = producer-attestation\ConvertTo-CanonicalJson $baselineEnvelope
    Assert-True ([Text.Encoding]::UTF8.GetString($producerBytes) -ceq $baselineCanonical) `
        'Dispatcher producer envelope is not exact shared canonical JSON.'
    $validNow = [DateTimeOffset]::Parse([string]$baselineEnvelope.payload.completed_at).AddSeconds(1)
    $null = producer-attestation\Test-CandidateProducerPayload -Payload $baselineEnvelope.payload `
        -CertificateId ([string]$baselineEnvelope.certificate.certificate_id) -Epoch 1 `
        -Scope 'candidate-evidence:exact-build' -Now $validNow
    $expiredNow = [DateTimeOffset]::Parse([string]$baselineEnvelope.payload.expires_at).AddTicks(1)
    $expiredRejected = $false
    try {
        $null = producer-attestation\Test-CandidateProducerPayload -Payload $baselineEnvelope.payload `
            -CertificateId ([string]$baselineEnvelope.certificate.certificate_id) -Epoch 1 `
            -Scope 'candidate-evidence:exact-build' -Now $expiredNow
    }
    catch { $expiredRejected = $_.Exception.Message -eq 'PRODUCER_ATTESTATION_EXPIRED' }
    Assert-True $expiredRejected 'An unchanged valid producer envelope remained valid after expiry.'

    $now = [DateTimeOffset]::UtcNow
    $producerMutations = @(
        @{ Name = 'missing exact field'; Apply = { param($v) $v.payload.PSObject.Properties.Remove('protocol_version') } },
        @{ Name = 'validly signed expired'; Apply = { param($v) $v.payload.started_at = $now.AddHours(-2).ToString('O'); $v.payload.completed_at = $now.AddMinutes(-90).ToString('O'); $v.payload.expires_at = $now.AddHours(-1).ToString('O') } },
        @{ Name = 'future issuance'; Apply = { param($v) $v.payload.started_at = $now.AddMinutes(10).ToString('O'); $v.payload.completed_at = $now.AddMinutes(11).ToString('O'); $v.payload.expires_at = $now.AddHours(1).ToString('O') } },
        @{ Name = 'future completion'; Apply = { param($v) $v.payload.started_at = $now.AddMinutes(-1).ToString('O'); $v.payload.completed_at = $now.AddHours(1).ToString('O'); $v.payload.expires_at = $now.AddHours(2).ToString('O') } },
        @{ Name = 'overlong lifetime'; Apply = { param($v) $v.payload.started_at = $now.AddHours(-1).ToString('O'); $v.payload.completed_at = $now.ToString('O'); $v.payload.expires_at = $now.AddHours(24).AddMinutes(1).ToString('O') } },
        @{ Name = 'cross-certificate identity'; Apply = { param($v) $v.payload.delegation_certificate_id = 'TEST-ONLY-retention-locator' } },
        @{ Name = 'protocol confusion'; Apply = { param($v) $v.payload.protocol_version = 'candidate-worker.v2' } },
        @{ Name = 'validly signed replay'; Apply = { param($v) $v.payload.run_id = 'replayed-signed-run' } }
    )
    foreach ($mutation in $producerMutations) {
        Write-ResignedProducerMutation $producerSourcePath $producerBytes $authority $mutation.Apply
        $mutationRoot = Join-Path $scratch ('reject-producer-' + ($mutation.Name -replace '[^a-z]+', '-'))
        $mutationOutput = @(& pwsh -NoProfile -File $testHarnessPath -EvidencePath $evidencePath `
            -BuildManifestPath $manifestPath -DestinationRoot $mutationRoot 2>&1)
        Assert-True ($LASTEXITCODE -ne 0) "Producer mutation '$($mutation.Name)' passed retention."
        if ($mutation.Name -eq 'future completion') {
            Assert-True (($mutationOutput -join ' | ') -match 'PRODUCER_CHRONOLOGY_INVALID') `
                'Future completion was not rejected by the signed chronology bound.'
        }
        Assert-True (-not (Test-Path -LiteralPath $mutationRoot)) `
            "Producer mutation '$($mutation.Name)' left a retained artifact."
    }
    [IO.File]::WriteAllBytes($producerSourcePath, $producerBytes)

    if ($Case -eq 'retention-admission') {
        $admissionHarnessPath = Join-Path (Split-Path -Parent $admissionPath) ('.x4-admission-test-' + [guid]::NewGuid().ToString('N') + '.ps1')
        $admissionSource = (Get-Content -LiteralPath $admissionPath -Raw).Replace(
            "`$retentionVerifierPath = Join-Path `$PSScriptRoot 'retain-evidence.ps1'",
            "`$retentionVerifierPath = '$($testHarnessPath.Replace("'", "''"))'"
        )
        Write-Utf8NoBom $admissionHarnessPath $admissionSource
        $admissionOutput = @(& pwsh -NoProfile -File $admissionHarnessPath `
            -DossierPath $dossierPath -RegistryPath $registryPath -CoveragePath $coveragePath `
            -FixturePath $fixturePath -VerifiedLocatorPath $locatorPath `
            -PendingLedgerPath $pendingLedgerPath -CandidateMatrixPath $matrixPath 2>&1)
        Assert-True ($LASTEXITCODE -eq 0) "Verified local chain did not reach the pending decision: $($admissionOutput -join ' | ')"
        $admissionResult = @($admissionOutput)[-1] | ConvertFrom-Json -Depth 16
        Assert-True ($admissionResult.verdict -eq 'chain-verified-x4-pending') 'Authenticated local evidence was not kept X4-pending.'

        $productionOutput = @(& pwsh -NoProfile -File $admissionPath `
            -DossierPath $dossierPath -RegistryPath $registryPath -CoveragePath $coveragePath `
            -FixturePath $fixturePath -VerifiedLocatorPath $locatorPath `
            -PendingLedgerPath $pendingLedgerPath -CandidateMatrixPath $matrixPath 2>&1)
        Assert-True ($LASTEXITCODE -ne 0) 'Production admission accepted the TEST-ONLY locator root.'
        Assert-True (($productionOutput -join "`n") -match 'RETENTION_ATTESTATION_UNCONFIGURED') 'Production TEST-root rejection was not stable.'
    }

    [byte[]]$retainedEvidenceBytes = [IO.File]::ReadAllBytes($retainedEvidencePath)
    [byte[]]$tamperedEvidenceBytes = [byte[]]$retainedEvidenceBytes.Clone()
    $tamperedEvidenceBytes[0] = $tamperedEvidenceBytes[0] -bxor 1
    [IO.File]::WriteAllBytes($retainedEvidencePath, $tamperedEvidenceBytes)
    $tamperedOutput = @(& pwsh -NoProfile -File $testHarnessPath -VerifyLocatorPath $locatorPath 2>&1)
    Assert-True ($LASTEXITCODE -ne 0) 'Changed retained bytes passed locator verification.'
    Assert-True (($tamperedOutput -join "`n") -match 'RETAINED_DIGEST_MISMATCH') 'Changed retained bytes returned an unstable status.'
    [IO.File]::WriteAllBytes($retainedEvidencePath, $retainedEvidenceBytes)

    $productionVerification = @(Invoke-Verification $locatorPath 1)
    Assert-True (($productionVerification -join "`n") -match 'RETENTION_ATTESTATION_UNCONFIGURED') 'Production accepted the TEST-ONLY root.'

    Add-BroadReadPermission $retainedEvidencePath
    $permissionOutput = Invoke-Verification $locatorPath 1
    Assert-True (($permissionOutput -join "`n") -match '"verdict":"rejected"') 'Permission mismatch did not block sanitized verification output.'

    if ($Case -eq 'retention-admission') {
        Write-Output 'PASS: retention-to-admission fail-closed contract'
        Write-Output 'PASS: retention-to-admission cryptographic core contract'
    }
    else { Write-Output 'PASS: evidence retention contract' }
}
finally {
    if ($null -ne $dispatcherHarnessPath -and (Test-Path -LiteralPath $dispatcherHarnessPath)) { Remove-Item -LiteralPath $dispatcherHarnessPath -Force }
    if ($null -ne $admissionHarnessPath -and (Test-Path -LiteralPath $admissionHarnessPath)) { Remove-Item -LiteralPath $admissionHarnessPath -Force }
    if ($null -ne $testHarnessPath -and (Test-Path -LiteralPath $testHarnessPath)) { Remove-Item -LiteralPath $testHarnessPath -Force }
    if ($null -ne $authority) {
        $authority.ProducerSigner.Dispose()
        $authority.RootSigner.Dispose()
        $authority.LocatorSigner.Dispose()
    }
    if (Test-Path -LiteralPath $reparseRoot) { [IO.Directory]::Delete($reparseRoot) }
    if (Test-Path -LiteralPath $scratch) {
        Remove-Item -LiteralPath $scratch -Recurse -Force
    }
}
