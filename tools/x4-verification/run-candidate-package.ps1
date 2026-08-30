[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateNotNullOrEmpty()][string]$GroupRoot,
    [Parameter(Mandatory)][ValidateNotNullOrEmpty()][string]$OutputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$dispatcherPath = $PSCommandPath
$launcherPath = Join-Path $repositoryRoot 'tools/x4-verification/isolation/invoke-candidate-worker.ps1'
$workerPath = Join-Path $repositoryRoot 'tools/x4-verification/isolation/candidate-worker.ps1'
$adapterPath = Join-Path $repositoryRoot 'tools/x4-verification/candidate-adapters.psm1'
$attestationModulePath = Join-Path $repositoryRoot 'tools/x4-verification/producer-attestation.psm1'
$protocolPath = Join-Path $repositoryRoot 'tools/x4-verification/contracts/candidate-worker-protocol.v1.json'
$schemaPath = Join-Path $repositoryRoot 'tools/x4-verification/contracts/runtime-evidence.v1.json'
$anchorPath = Join-Path $repositoryRoot 'tools/x4-verification/contracts/owner-root-anchor.v1.json'
$trustedComponentBindings = [ordered]@{
    dispatcher_digest = $dispatcherPath
    adapter_digest = $adapterPath
    attestation_module_digest = $attestationModulePath
    worker_digest = $workerPath
    launcher_digest = $launcherPath
    worker_protocol_digest = $protocolPath
    runtime_evidence_schema_digest = $schemaPath
    owner_root_anchor_digest = $anchorPath
}
$script:ProductionRootSpkiSha256 = 'UNCONFIGURED'
$script:ProducerPurpose = 'candidate-producer'
$script:ProducerScope = 'candidate-evidence:exact-build'
$script:ProducerEpoch = 1
$script:SignatureAlgorithm = 'ECDSA_P256_SHA256'
$script:TestOnlyHarness = $false
$script:TestAuthorityPath = ''
$script:ReasonCode = 'DISPATCH_INTERNAL_FAILURE'

function Fail([string]$Code) { $script:ReasonCode = $Code; throw [IO.InvalidDataException]::new($Code) }
function Get-Sha256([byte[]]$Bytes) { [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant() }
function Get-FileDigest([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { Fail 'COMPONENT_MISSING' }
    Get-Sha256 ([IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Path).Path))
}
function Test-Contained([string]$Path, [string]$Root) {
    $full = [IO.Path]::GetFullPath($Path).TrimEnd('\', '/')
    $base = [IO.Path]::GetFullPath($Root).TrimEnd('\', '/')
    $full.Equals($base, [StringComparison]::OrdinalIgnoreCase) -or $full.StartsWith($base + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)
}
function Assert-SafeOutputDestination([string]$Path) {
    $full = [IO.Path]::GetFullPath($Path)
    if (Test-Contained $full $repositoryRoot) { Fail 'OUTPUT_DESTINATION_REJECTED' }
    if ($full -match '(?i)[\\/]steamapps[\\/]common[\\/]X4 Foundations(?:[\\/]|$)' -or
        $full -match '(?i)[\\/]X4 Foundations[\\/]extensions(?:[\\/]|$)') {
        Fail 'GAME_INSTALLATION_DESTINATION_REJECTED'
    }
    if ($full -match '(?i)[\\/]extensions[\\/]live_galaxy(?:[\\/]|$)') {
        Fail 'PUBLIC_RUNTIME_DESTINATION_REJECTED'
    }
    if ($full -match '(?i)[\\/]Egosoft[\\/]X4[\\/][0-9]+[\\/]save(?:[\\/]|$)') {
        Fail 'GAME_SAVE_DESTINATION_REJECTED'
    }
}
function Assert-NoReparse([string]$Path) {
    $full = [IO.Path]::GetFullPath($Path)
    $root = [IO.Path]::GetPathRoot($full)
    $current = $root
    foreach ($segment in @($full.Substring($root.Length) -split '[\\/]+')) {
        if ([string]::IsNullOrEmpty($segment)) { continue }
        $current = Join-Path $current $segment
        if (-not (Test-Path -LiteralPath $current)) { break }
        if (((Get-Item -LiteralPath $current -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { Fail 'REPARSE_PATH_REJECTED' }
    }
}
function Set-OwnerOnly([string]$Path, [bool]$Directory = $false) {
    if (-not $IsWindows) {
        & chmod $(if ($Directory) { '700' } else { '600' }) -- $Path
        if ($LASTEXITCODE -ne 0) { Fail 'OUTPUT_PERMISSION_FAILED' }
        return
    }
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent().User
    $grant = if ($Directory) { "*$($identity.Value):(OI)(CI)F" } else { "*$($identity.Value):F" }
    $null = & icacls.exe $Path /inheritance:r /grant:r $grant
    if ($LASTEXITCODE -ne 0) { Fail 'OUTPUT_PERMISSION_FAILED' }
}
function Assert-OwnerOnly([string]$Path) {
    if (-not $IsWindows) {
        $mode = [IO.File]::GetUnixFileMode($Path)
        $broad = [IO.UnixFileMode]::GroupWrite -bor [IO.UnixFileMode]::OtherWrite
        if (($mode -band $broad) -ne 0) { Fail 'OWNER_ONLY_PATH_REQUIRED' }
        return
    }
    $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    $acl = Get-Acl -LiteralPath $Path
    $rules = @($acl.GetAccessRules($true, $true, [Security.Principal.SecurityIdentifier]))
    if (-not $acl.AreAccessRulesProtected -or
        @($rules | Where-Object {
            $_.AccessControlType -eq [Security.AccessControl.AccessControlType]::Allow -and
            $_.IdentityReference.Value -ne $sid
        }).Count -ne 0) { Fail 'OWNER_ONLY_PATH_REQUIRED' }
}
function New-VerifiedSnapshot([string]$SourceRoot, $Manifest, [string]$WorkRoot) {
    $snapshot = Join-Path $WorkRoot 'verified-snapshot'
    $null = New-Item -ItemType Directory -Path $snapshot
    Set-OwnerOnly $snapshot $true
    foreach ($file in @($Manifest.generated_files)) {
        $source = [IO.Path]::GetFullPath((Join-Path $SourceRoot ([string]$file.path)))
        if (-not (Test-Contained $source $SourceRoot)) { Fail 'SNAPSHOT_PATH_ESCAPE' }
        Assert-NoReparse $source
        [byte[]]$bytes = [IO.File]::ReadAllBytes($source)
        if ($bytes.Length -ne [long]$file.bytes -or (Get-Sha256 $bytes) -ne [string]$file.sha256) {
            Fail 'COMPONENT_DIGEST_MISMATCH'
        }
        $destination = [IO.Path]::GetFullPath((Join-Path $snapshot ([string]$file.path)))
        if (-not (Test-Contained $destination $snapshot)) { Fail 'SNAPSHOT_PATH_ESCAPE' }
        $parent = [IO.Path]::GetDirectoryName($destination)
        if (-not (Test-Path -LiteralPath $parent)) {
            $null = New-Item -ItemType Directory -Path $parent -Force
        }
        Set-OwnerOnly $parent $true
        [IO.File]::WriteAllBytes($destination, $bytes)
        Set-OwnerOnly $destination
        if ((Get-FileDigest $destination) -ne [string]$file.sha256) {
            Fail 'SNAPSHOT_DIGEST_MISMATCH'
        }
    }
    return $snapshot
}
function Write-Result([string]$Verdict, [bool]$Ready, [string]$Attestation, [string]$Digest = '', [bool]$Retainable = $false) {
    [ordered]@{
        schema_version = 'candidate-package-run.v1'; verdict = $Verdict
        reason_code = $script:ReasonCode; local_process_ready = $Ready
        evidence_classification = 'authenticated-local-contract'
        retainable = $Retainable; attestation_status = $Attestation; evidence_digest = $Digest
    } | ConvertTo-Json -Compress
}

function Get-LocalContractVector([string]$CandidateId) {
    switch ($CandidateId) {
        'p051-cadence-seta' {
            return [pscustomobject]@{ Expected = 'cadence:seta-ratio=6'; Fixture = [ordered]@{ real_ms = 1000; game_ms = 6000; seta_active = $true; sample_count = 4 } }
        }
        'p051-lifecycle-reload' {
            return [pscustomobject]@{ Expected = 'lifecycle:single-registration'; Fixture = [ordered]@{ registration_ids = @('live-galaxy-candidate'); reload_count = 1 } }
        }
        'p051-mod-stack-compatibility' {
            return [pscustomobject]@{ Expected = 'mod-stack:declared-coexistence'; Fixture = [ordered]@{ enabled_mod_ids = @('kuda-ai-tweaks', 'more-ai-economy-ships', 'add-more-sectors'); excluded_mod_ids = @('faction-enhancer') } }
        }
        'p051-native-count-fill-runtime' {
            return [pscustomobject]@{ Expected = 'count-fill:3-of-3'; Fixture = [ordered]@{ reported_count = 3; records = @('alpha', 'beta', 'gamma') } }
        }
        'p051-native-fill-completeness' {
            return [pscustomobject]@{ Expected = 'fill:complete=3'; Fixture = [ordered]@{ requested_count = 3; returned_count = 3; records = @('alpha', 'beta', 'gamma') } }
        }
        'p051-native-identity-closure' {
            return [pscustomobject]@{ Expected = 'identity:object=station-01/owner=argon'; Fixture = [ordered]@{ native_id = 'station-01'; canonical_id = 'station-01'; owner_id = 'argon'; canonical_owner_id = 'argon' } }
        }
        'p051-native-volume-envelope' {
            return [pscustomobject]@{ Expected = 'volume:8-samples/2048-bytes'; Fixture = [ordered]@{ sample_count = 8; max_samples = 16; payload_bytes = 2048; max_payload_bytes = 4096 } }
        }
        default { Fail 'ADAPTER_ID_REJECTED' }
    }
}

function Read-BoundedJson([string]$Path, [int]$MaximumBytes = 32768) {
    Assert-NoReparse $Path
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw 'PRODUCER_ATTESTATION_UNCONFIGURED' }
    $item = Get-Item -LiteralPath $Path -Force
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or $item.Length -gt $MaximumBytes) { throw 'PRODUCER_AUTHORITY_INVALID' }
    return Get-Content -LiteralPath $Path -Raw -Encoding utf8 | ConvertFrom-Json -Depth 32 -DateKind String
}

function ConvertTo-CertificateBytes($Certificate) {
    $fields = @('schema_version', 'certificate_id', 'root_id', 'root_spki_sha256', 'delegated_spki_sha256', 'windows_key_name', 'purpose', 'epoch', 'scope', 'algorithm', 'not_before', 'not_after', 'policy_digest')
    $builder = [Text.StringBuilder]::new()
    foreach ($field in $fields) {
        if ($Certificate.PSObject.Properties.Name -notcontains $field) { throw 'PRODUCER_CERTIFICATE_INVALID' }
        $value = [string]$Certificate.$field
        [void]$builder.Append($field).Append('=').Append($value.Length).Append(':').Append($value).Append("`n")
    }
    return [Text.Encoding]::UTF8.GetBytes($builder.ToString())
}

function Get-ProducerSigningAuthority {
    if ($script:TestOnlyHarness) {
        if ([string]::IsNullOrWhiteSpace($script:TestAuthorityPath)) { throw 'TEST_AUTHORITY_PATH_REJECTED' }
        $authority = Read-BoundedJson $script:TestAuthorityPath 65536
        if ($authority.schema_version -ne 'retention-test-authority.v1' -or
            $authority.marker -ne 'TEST-ONLY-NEVER-PRODUCTION' -or
            $null -eq $authority.producer_certificate -or
            [string]::IsNullOrWhiteSpace([string]$authority.producer_private_pkcs8_base64)) {
            throw 'TEST_AUTHORITY_INVALID'
        }
        [byte[]]$privateBytes = [Convert]::FromBase64String(
            [string]$authority.producer_private_pkcs8_base64
        )
        $signer = [Security.Cryptography.ECDsa]::Create()
        $read = 0
        $signer.ImportPkcs8PrivateKey($privateBytes, [ref]$read)
        return [pscustomobject]@{
            ready = $true; status = 'PRODUCER_ATTESTATION_VERIFIED'
            certificate = $authority.producer_certificate; key = $null; signer = $signer
        }
    }
    if (-not $IsWindows) { return [pscustomobject]@{ ready = $false; status = 'PRODUCER_ATTESTATION_PLATFORM_UNSUPPORTED' } }
    try {
        $anchor = Read-BoundedJson $anchorPath
        if ($anchor.schema_version -ne 'x4-owner-root-anchor.v1' -or $anchor.root_id -ne 'live-galaxy-owner-root-v1' -or
            $anchor.algorithm -ne $script:SignatureAlgorithm -or $anchor.policy_digest -ne '6497cd18a4ee0286f0d566978c9225eaa168ba5d71f8fd7042a78507e1462854' -or
            [int]$anchor.accepted_epochs.'candidate-producer' -ne $script:ProducerEpoch -or
            [string]$anchor.scopes.'candidate-producer' -ne $script:ProducerScope) { throw 'PRODUCER_ROOT_PIN_MISMATCH' }
        if ($anchor.status -eq 'unconfigured') { return [pscustomobject]@{ ready = $false; status = 'PRODUCER_ATTESTATION_UNCONFIGURED' } }
        [byte[]]$rootSpki = [Convert]::FromBase64String([string]$anchor.root_spki_der_base64)
        if ($anchor.status -ne 'configured' -or $script:ProductionRootSpkiSha256 -eq 'UNCONFIGURED' -or
            $anchor.root_spki_sha256 -ne $script:ProductionRootSpkiSha256 -or (Get-Sha256 $rootSpki) -ne $script:ProductionRootSpkiSha256) { throw 'PRODUCER_ROOT_PIN_MISMATCH' }
        $certificatePath = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'LiveGalaxy/authority/candidate-producer-certificate.v1.json'
        $certificate = Read-BoundedJson $certificatePath
        $policyDigest = Get-Sha256 ([Text.Encoding]::UTF8.GetBytes("$($script:ProducerPurpose)|$($script:ProducerEpoch)|$($script:ProducerScope)"))
        if ($certificate.schema_version -ne 'x4-delegated-purpose-certificate.v1' -or $certificate.root_id -ne $anchor.root_id -or
            $certificate.root_spki_sha256 -ne $script:ProductionRootSpkiSha256 -or $certificate.purpose -ne $script:ProducerPurpose -or
            [int]$certificate.epoch -ne $script:ProducerEpoch -or $certificate.scope -ne $script:ProducerScope -or
            $certificate.algorithm -ne $script:SignatureAlgorithm -or $certificate.policy_digest -ne $policyDigest) { throw 'PRODUCER_CERTIFICATE_POLICY_MISMATCH' }
        $now = [DateTimeOffset]::UtcNow
        if ($now -lt [DateTimeOffset]::Parse([string]$certificate.not_before) -or $now -ge [DateTimeOffset]::Parse([string]$certificate.not_after)) { throw 'PRODUCER_CERTIFICATE_EXPIRED' }
        [byte[]]$delegatedSpki = [Convert]::FromBase64String([string]$certificate.delegated_spki_der_base64)
        if ((Get-Sha256 $delegatedSpki) -ne $certificate.delegated_spki_sha256) { throw 'PRODUCER_DELEGATED_KEY_MISMATCH' }
        [byte[]]$rootSignature = [Convert]::FromBase64String([string]$certificate.root_signature_base64)
        $rootVerifier = [Security.Cryptography.ECDsa]::Create()
        try {
            $read = 0; $rootVerifier.ImportSubjectPublicKeyInfo($rootSpki, [ref]$read)
            if (-not $rootVerifier.VerifyData((ConvertTo-CertificateBytes $certificate), $rootSignature, [Security.Cryptography.HashAlgorithmName]::SHA256, [Security.Cryptography.DSASignatureFormat]::IeeeP1363FixedFieldConcatenation)) { throw 'PRODUCER_CERTIFICATE_SIGNATURE_INVALID' }
        }
        finally { $rootVerifier.Dispose() }
        $key = [Security.Cryptography.CngKey]::Open([string]$certificate.windows_key_name, [Security.Cryptography.CngProvider]::MicrosoftSoftwareKeyStorageProvider, [Security.Cryptography.CngKeyOpenOptions]::UserKey)
        if (-not (producer-attestation\Test-CngSigningKeyPolicy $key)) {
            $key.Dispose(); throw 'PRODUCER_DELEGATED_KEY_USER_PRESENCE_REQUIRED'
        }
        $signer = [Security.Cryptography.ECDsaCng]::new($key)
        if ((Get-Sha256 $signer.ExportSubjectPublicKeyInfo()) -ne $certificate.delegated_spki_sha256) { $signer.Dispose(); $key.Dispose(); throw 'PRODUCER_DELEGATED_KEY_MISMATCH' }
        return [pscustomobject]@{ ready = $true; status = 'PRODUCER_ATTESTATION_VERIFIED'; certificate = $certificate; key = $key; signer = $signer }
    }
    catch [Security.Cryptography.CryptographicException] { return [pscustomobject]@{ ready = $false; status = 'PRODUCER_ATTESTATION_UNCONFIGURED' } }
    catch { return [pscustomobject]@{ ready = $false; status = [string]$_.Exception.Message } }
}

function New-ProducerAttestation($Manifest, $Subset, $Rows, [string]$RunId, [string]$EvidenceDigest, [string]$StartedAt, [string]$CompletedAt, [string]$OutputFull) {
    $authority = Get-ProducerSigningAuthority
    if (-not $authority.ready) { return $authority }
    try {
        $payload = [ordered]@{
            schema_version = 'candidate-producer-envelope.v1'; authority_purpose = $script:ProducerPurpose
            delegation_certificate_id = $authority.certificate.certificate_id; protocol_version = 'candidate-worker.v1'
            purpose = $script:ProducerPurpose; epoch = $script:ProducerEpoch; scope = $script:ProducerScope
            dispatcher_digest = $Manifest.dispatcher_digest; adapter_digest = $Manifest.adapter_digest; worker_digest = $Manifest.worker_digest
            attestation_module_digest = $Manifest.attestation_module_digest
            launcher_digest = $Manifest.launcher_digest; worker_protocol_digest = $Manifest.worker_protocol_digest
            runtime_evidence_schema_digest = $Manifest.runtime_evidence_schema_digest; build_id = $Manifest.build_id
            package_conformance_digest = $Manifest.package_conformance_digest; matrix_digest = $Manifest.matrix_digest
            run_id = $RunId; candidate_ids = @($Subset.candidates.id | Sort-Object); evidence_digest = $EvidenceDigest
            started_at = $StartedAt; completed_at = $CompletedAt; classification = 'authenticated-local-contract'
            nonce = [guid]::NewGuid().ToString('N'); expires_at = ([DateTimeOffset]::Parse($StartedAt)).AddHours(24).ToString('O')
            signature_algorithm = $script:SignatureAlgorithm
        }
        $envelope = New-CandidateProducerAttestation -Payload $payload `
            -Certificate $authority.certificate -Signer $authority.signer
        [byte[]]$envelopeBytes = [Text.UTF8Encoding]::new($false).GetBytes(
            (ConvertTo-CanonicalJson $envelope)
        )
        if ($envelopeBytes.Length -gt 65536) { throw 'PRODUCER_ENVELOPE_BOUND_EXCEEDED' }
        $attestationPath = "$OutputFull.attestation.json"
        if (Test-Path -LiteralPath $attestationPath) { throw 'PRODUCER_ATTESTATION_DESTINATION_EXISTS' }
        $temporary = "$attestationPath.$PID.tmp"; [IO.File]::WriteAllBytes($temporary, $envelopeBytes); Set-OwnerOnly $temporary; [IO.File]::Move($temporary, $attestationPath)
        return [pscustomobject]@{ ready = $true; status = 'PRODUCER_ATTESTATION_VERIFIED' }
    }
    finally {
        $authority.signer.Dispose()
        if ($null -ne $authority.key) { $authority.key.Dispose() }
    }
}

$workRoot = $null
try {
    $groupFull = [IO.Path]::GetFullPath($GroupRoot)
    $outputFull = [IO.Path]::GetFullPath($OutputPath)
    $script:ReasonCode = 'PATH_VALIDATION_FAILED'
    if (Test-Contained $dispatcherPath $groupFull) { Fail 'UNTRUSTED_DISPATCHER_ORIGIN' }
    Assert-NoReparse $groupFull
    Assert-NoReparse ([IO.Path]::GetDirectoryName($outputFull))
    if (-not (Test-Path -LiteralPath $groupFull -PathType Container) -or (Test-Path -LiteralPath $outputFull)) { Fail 'DISPATCH_PATH_INVALID' }
    Assert-SafeOutputDestination $outputFull
    Assert-OwnerOnly $groupFull
    Assert-OwnerOnly ([IO.Path]::GetDirectoryName($outputFull))
    $manifestPath = Join-Path $groupFull 'manifest/build-manifest.v1.json'
    $script:ReasonCode = 'MANIFEST_VALIDATION_FAILED'
    Assert-NoReparse $manifestPath
    [byte[]]$manifestBytes = [IO.File]::ReadAllBytes($manifestPath)
    $manifest = [Text.Encoding]::UTF8.GetString($manifestBytes) | ConvertFrom-Json -Depth 64 -DateKind String
    if ($manifest.execution_status -ne 'execution-ready-local-process' -or
        $manifest.native_execution_status -ne 'terminable-external-isolation' -or
        $manifest.local_readiness_verified -ne $true) { Fail 'READINESS_STATUS_INVALID' }
    $workRoot = Join-Path ([IO.Path]::GetTempPath()) ("live-galaxy-dispatch-" + [guid]::NewGuid().ToString('N'))
    $null = New-Item -ItemType Directory -Path $workRoot
    Set-OwnerOnly $workRoot $true
    $sourceGroupFull = $groupFull
    $groupFull = New-VerifiedSnapshot $sourceGroupFull $manifest $workRoot
    $adapterPath = Join-Path $groupFull 'tools/x4-verification/candidate-adapters.psm1'
    $attestationModulePath = Join-Path $groupFull 'tools/x4-verification/producer-attestation.psm1'
    $workerPath = Join-Path $groupFull 'tools/x4-verification/isolation/candidate-worker.ps1'
    $launcherPath = Join-Path $groupFull 'tools/x4-verification/isolation/invoke-candidate-worker.ps1'
    $protocolPath = Join-Path $groupFull 'tools/x4-verification/contracts/candidate-worker-protocol.v1.json'
    $schemaPath = Join-Path $groupFull 'tools/x4-verification/contracts/runtime-evidence.v1.json'
    $anchorPath = Join-Path $groupFull 'tools/x4-verification/contracts/owner-root-anchor.v1.json'
    Import-Module $attestationModulePath -Force
    $subsetPath = Join-Path $groupFull 'manifest/candidate-matrix-subset.v1.json'
    $subset = Get-Content -LiteralPath $subsetPath -Raw | ConvertFrom-Json -Depth 64 -DateKind String
    $candidateIds = @($subset.candidates.id | Sort-Object)
    if ($candidateIds.Count -lt 1 -or $candidateIds.Count -gt 7 -or
        @($candidateIds | Sort-Object -Unique).Count -ne $candidateIds.Count -or
        ($candidateIds -join '|') -ne (@($manifest.candidate_ids | Sort-Object) -join '|')) { Fail 'CANDIDATE_SET_INVALID' }
    $script:ReasonCode = 'COMPONENT_BINDING_VALIDATION_FAILED'
    $componentBindings = [ordered]@{
        dispatcher_digest = (Join-Path $groupFull 'tools/x4-verification/run-candidate-package.ps1')
        adapter_digest = $adapterPath
        attestation_module_digest = $attestationModulePath
        worker_digest = $workerPath
        launcher_digest = $launcherPath
        worker_protocol_digest = $protocolPath; runtime_evidence_schema_digest = $schemaPath
        owner_root_anchor_digest = $anchorPath
    }
    foreach ($binding in $componentBindings.GetEnumerator()) {
        $bindingName = [string]$binding.Key
        $bindingPath = [string]$binding.Value
        $property = $manifest.PSObject.Properties[$bindingName]
        $trustedPath = [string]$trustedComponentBindings[$bindingName]
        $componentDigest = Get-FileDigest $bindingPath
        $trustedDigest = Get-FileDigest $trustedPath
        if ($null -eq $property -or [string]$property.Value -ne $componentDigest -or
            $componentDigest -ne $trustedDigest) {
            Fail 'COMPONENT_DIGEST_MISMATCH'
        }
    }
    $script:ReasonCode = 'PACKAGE_CONFORMANCE_VALIDATION_FAILED'
    $contentPath = Join-Path $groupFull 'content.xml'
    $uiPath = Join-Path $groupFull 'ui.xml'
    $entryPath = Join-Path $groupFull 'lua/live_galaxy_candidate_entry.lua'
    $graphDigest = Get-Sha256 ([Text.Encoding]::UTF8.GetBytes(((Get-FileDigest $contentPath) + (Get-FileDigest $uiPath) + (Get-FileDigest $entryPath))))
    $packageFields = @('schema_version', 'verdict', 'classification', 'evidence_level', 'graph_digest', 'dossier_digest', 'coverage_digest')
    if ((@($manifest.package_conformance.PSObject.Properties.Name | Sort-Object) -join '|') -ne (@($packageFields | Sort-Object) -join '|') -or
        $manifest.package_conformance.verdict -ne 'conformant' -or
        $manifest.package_conformance.classification -ne 'local-only' -or
        $manifest.package_conformance.evidence_level -ne 'packaged-static' -or
        $manifest.package_conformance.graph_digest -ne $graphDigest) { Fail 'PACKAGE_CONFORMANCE_MISMATCH' }
    $packageBytes = [Text.Encoding]::UTF8.GetBytes(($manifest.package_conformance | ConvertTo-Json -Compress -Depth 8))
    if ($manifest.package_conformance_digest -ne (Get-Sha256 $packageBytes)) { Fail 'PACKAGE_CONFORMANCE_MISMATCH' }
    $script:ReasonCode = 'ADAPTER_VALIDATION_FAILED'
    Import-Module $adapterPath -Force
    $knownIds = @(Get-CandidateAdapterDefinitions).id
    if (@($candidateIds | Where-Object { $knownIds -cnotcontains $_ }).Count -ne 0) { Fail 'ADAPTER_ID_REJECTED' }

    $runId = 'local-' + [guid]::NewGuid().ToString('N')
    $startedAt = [DateTimeOffset]::UtcNow.ToString('O')
    $rows = @()
    foreach ($candidate in @($subset.candidates | Sort-Object id)) {
        $vector = Get-LocalContractVector ([string]$candidate.id)
        $requestPath = Join-Path $workRoot "$($candidate.id).request.json"
        $responsePath = Join-Path $workRoot "$($candidate.id).response.json"
        $request = [ordered]@{
            schema_version = 'candidate-worker.v1'; request_id = [guid]::NewGuid().ToString('N')
            run_id = $runId; candidate_id = $candidate.id; adapter_id = 'local-contract-success'
            issued_at_utc = [DateTimeOffset]::UtcNow.ToString('O')
            input = [ordered]@{ expected_result = $vector.Expected; fixture = $vector.Fixture; max_work_units = 8 }
        }
        [IO.File]::WriteAllText($requestPath, ($request | ConvertTo-Json -Compress -Depth 8), [Text.UTF8Encoding]::new($false))
        $launchText = & pwsh -NoProfile -File $launcherPath -RequestPath $requestPath -ResponsePath $responsePath -DeadlineMs 2000
        $launch = $launchText | ConvertFrom-Json -Depth 16 -DateKind String
        $execution = if ($launch.accepted) { 'pass' } else { 'fail' }
        $contract = if ($launch.accepted -and $launch.response.completeness -eq 'complete') { 'pass' } else { 'fail' }
        $effect = if ($contract -eq 'pass' -and $launch.response.actual_result -ceq $vector.Expected) { 'pass' } else { 'mismatch' }
        $rows += [ordered]@{
            schema_version = 'candidate-local-evidence.v1'; run_id = $runId; candidate_id = $candidate.id
            build_id = $manifest.build_id; group_id = $manifest.group_id; execution_verdict = $execution
            contract_verdict = $contract; effect_verdict = $effect; completeness = if ($launch.accepted) { $launch.response.completeness } else { 'incomplete' }
            diagnostic_code = $launch.diagnostic_code; evidence_classification = 'authenticated-local-contract'
        }
    }
    if (@($rows | Where-Object { $_.execution_verdict -ne 'pass' -or $_.contract_verdict -ne 'pass' -or $_.effect_verdict -ne 'pass' }).Count -ne 0) { Fail 'CANDIDATE_RUN_INCOMPLETE' }
    $text = (@($rows | ForEach-Object { $_ | ConvertTo-Json -Compress -Depth 8 }) -join "`n") + "`n"
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($text)
    if ($bytes.Length -gt 65536) { Fail 'OUTPUT_BOUND_EXCEEDED' }
    $temporary = "$outputFull.$PID.tmp"
    Assert-NoReparse ([IO.Path]::GetDirectoryName($outputFull))
    Assert-SafeOutputDestination $outputFull
    [IO.File]::WriteAllBytes($temporary, $bytes)
    Set-OwnerOnly $temporary
    Assert-NoReparse ([IO.Path]::GetDirectoryName($outputFull))
    Assert-SafeOutputDestination $outputFull
    [IO.File]::Move($temporary, $outputFull)
    $digest = Get-FileDigest $outputFull
    $completedAt = [DateTimeOffset]::UtcNow.ToString('O')
    $attestation = New-ProducerAttestation $manifest $subset $rows $runId $digest $startedAt $completedAt $outputFull
    $script:ReasonCode = $attestation.status
    Write-Result 'pass' $true $attestation.status $digest $attestation.ready
    exit 0
}
catch {
    Write-Verbose "$script:ReasonCode / $($_.Exception.Message) at line $($_.InvocationInfo.ScriptLineNumber)"
    Write-Result 'fail' $false 'PRODUCER_ATTESTATION_UNCONFIGURED'
    exit 1
}
finally {
    if ($null -ne $workRoot -and (Test-Path -LiteralPath $workRoot)) { Remove-Item -LiteralPath $workRoot -Recurse -Force }
}
