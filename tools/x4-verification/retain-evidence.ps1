[CmdletBinding(DefaultParameterSetName = 'Retain')]
param(
    [Parameter(Mandatory = $true, ParameterSetName = 'Retain')]
    [ValidateNotNullOrEmpty()]
    [string]$EvidencePath,
    [Parameter(Mandatory = $true, ParameterSetName = 'Retain')]
    [ValidateNotNullOrEmpty()]
    [string]$BuildManifestPath,
    [Parameter(Mandatory = $true, ParameterSetName = 'Retain')]
    [ValidateNotNullOrEmpty()]
    [string]$DestinationRoot,
    [Parameter(Mandatory = $true, ParameterSetName = 'Verify')]
    [ValidateNotNullOrEmpty()]
    [string]$VerifyLocatorPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$runtimeContractPath = Join-Path $PSScriptRoot 'contracts/runtime-evidence.v1.json'
$manifestContractPath = Join-Path $PSScriptRoot 'contracts/candidate-build-manifest.v1.json'
$sanitizedContractPath = Join-Path $PSScriptRoot 'contracts/sanitized-ledger.v1.json'
$matrixPath = Join-Path $repositoryRoot 'tests/x4-candidates/phase-05.1-candidates.v1.json'
$dossierPath = Join-Path $PSScriptRoot 'contracts/phase-05.1-dossier.v1.json'
$registryPath = Join-Path $PSScriptRoot 'contracts/known-failures.v1.json'
$coveragePath = Join-Path $PSScriptRoot 'contracts/coverage.v1.json'
$packageConformancePath = Join-Path $PSScriptRoot 'x4-package-conformance.ps1'
$publicPackageRoot = Join-Path $repositoryRoot 'extensions/live_galaxy'
$ownerRootAnchorPath = Join-Path $PSScriptRoot 'contracts/owner-root-anchor.v1.json'
$producerModulePath = Join-Path $PSScriptRoot 'producer-attestation.psm1'
$script:ProductionRootSpkiSha256 = 'UNCONFIGURED'
$script:TestOnlyHarness = $false
$script:SimulateUnsupportedPlatform = $false
$script:TestAuthorityPath = ''
$script:SignatureAlgorithm = 'ECDSA_P256_SHA256'
$script:ProducerPurpose = 'candidate-producer'
$script:ProducerScope = 'candidate-evidence:exact-build'
$script:LocatorPurpose = 'retention-locator'
$script:LocatorScope = 'retained-evidence:exact-run'

$script:failureCode = 'INTERNAL_RETENTION_ERROR'
$script:diagnosticId = 'startup'
$script:cleanupRoot = $null
$script:cleanupDestination = $null
$script:createdDestination = $false

Import-Module $producerModulePath -Force

function Fail([string]$Code) {
    $script:failureCode = $Code
    throw [InvalidOperationException]::new($Code)
}

function Require-Property($Value, [string]$Name) {
    if ($null -eq $Value -or $Value.PSObject.Properties.Name -notcontains $Name) {
        Fail 'MISSING_REQUIRED_FIELD'
    }
    return $Value.$Name
}

function Require-Text($Value, [int]$Maximum = 256) {
    if ($Value -isnot [string] -or [string]::IsNullOrWhiteSpace($Value) -or
        [Text.Encoding]::UTF8.GetByteCount($Value) -gt $Maximum) {
        Fail 'INVALID_FIELD_VALUE'
    }
}

function Require-Id($Value) {
    Require-Text $Value 128
    if ($Value -notmatch '^[a-z0-9][a-z0-9._-]{0,127}$') { Fail 'INVALID_ID' }
}

function Require-Digest($Value) {
    if ($Value -isnot [string] -or $Value -notmatch '^[a-f0-9]{64}$') {
        Fail 'INVALID_DIGEST'
    }
}

function Require-Array($Value, [int]$Maximum, [int]$Minimum = 0) {
    if ($null -eq $Value -or $Value -is [string]) { Fail 'INVALID_FIELD_VALUE' }
    $items = @($Value)
    if ($items.Count -lt $Minimum -or $items.Count -gt $Maximum) { Fail 'BOUND_EXCEEDED' }
    return $items
}

function Require-Integer($Value, [long]$Maximum = [long]::MaxValue) {
    if ($Value -isnot [byte] -and $Value -isnot [int16] -and $Value -isnot [int32] -and $Value -isnot [int64]) {
        Fail 'INVALID_FIELD_VALUE'
    }
    if ([long]$Value -lt 0 -or [long]$Value -gt $Maximum) { Fail 'BOUND_EXCEEDED' }
}

function Get-Sha256Bytes([byte[]]$Bytes) {
    return ([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes))).ToLowerInvariant()
}

function Get-Sha256File([string]$Path) {
    return Get-Sha256Bytes ([IO.File]::ReadAllBytes($Path))
}

function Read-BoundedBytes([string]$Path, [int]$Maximum) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { Fail 'MISSING_INPUT' }
    $item = Get-Item -LiteralPath $Path
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { Fail 'REPARSE_POINT_REJECTED' }
    $bytes = [IO.File]::ReadAllBytes($item.FullName)
    if ($bytes.Length -gt $Maximum) { Fail 'BOUND_EXCEEDED' }
    return $bytes
}

function Read-BoundedJson([string]$Path, [int]$Maximum, [string]$SchemaVersion) {
    $bytes = Read-BoundedBytes $Path $Maximum
    try { $value = [Text.Encoding]::UTF8.GetString($bytes) | ConvertFrom-Json -Depth 64 -DateKind String }
    catch { Fail 'MALFORMED_JSON' }
    if ((Require-Property $value 'schema_version') -ne $SchemaVersion) { Fail 'UNSUPPORTED_SCHEMA' }
    return [pscustomobject]@{ Value = $value; Bytes = $bytes }
}

function ConvertTo-CanonicalValue($Value) {
    if ($null -eq $Value -or $Value -is [string] -or $Value -is [bool] -or
        $Value -is [byte] -or $Value -is [int16] -or $Value -is [int32] -or
        $Value -is [int64] -or $Value -is [decimal] -or $Value -is [double]) {
        return $Value
    }
    if ($Value -is [Collections.IDictionary]) {
        $dictionary = [ordered]@{}
        foreach ($key in @($Value.Keys | Sort-Object)) {
            $dictionary[$key] = ConvertTo-CanonicalValue $Value[$key]
        }
        return $dictionary
    }
    if ($Value -is [Collections.IEnumerable] -and $Value -isnot [pscustomobject]) {
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

function ConvertTo-CertificateBytes($Certificate) {
    $fields = @(
        'schema_version', 'certificate_id', 'root_id', 'root_spki_sha256',
        'delegated_spki_sha256', 'windows_key_name', 'purpose', 'epoch',
        'scope', 'algorithm', 'not_before', 'not_after', 'policy_digest'
    )
    $builder = [Text.StringBuilder]::new()
    foreach ($field in $fields) {
        if ($Certificate.PSObject.Properties.Name -notcontains $field) { Fail 'RETENTION_CERTIFICATE_INVALID' }
        $value = [string]$Certificate.$field
        [void]$builder.Append($field).Append('=').Append($value.Length).Append(':').Append($value).Append("`n")
    }
    return [Text.Encoding]::UTF8.GetBytes($builder.ToString())
}

function Get-RootContext {
    if ($script:SimulateUnsupportedPlatform -or (-not $IsWindows -and -not $script:TestOnlyHarness)) {
        Fail 'RETENTION_ATTESTATION_PLATFORM_UNSUPPORTED'
    }
    $anchor = (Read-BoundedJson $ownerRootAnchorPath 32768 'x4-owner-root-anchor.v1').Value
    if ($anchor.root_id -ne 'live-galaxy-owner-root-v1' -or $anchor.algorithm -ne $script:SignatureAlgorithm -or
        $anchor.policy_digest -ne '6497cd18a4ee0286f0d566978c9225eaa168ba5d71f8fd7042a78507e1462854') {
        Fail 'RETENTION_ROOT_PIN_MISMATCH'
    }
    if ($anchor.status -eq 'unconfigured') { Fail 'RETENTION_ATTESTATION_UNCONFIGURED' }
    [byte[]]$rootSpki = [Convert]::FromBase64String([string]$anchor.root_spki_der_base64)
    if ($anchor.status -ne 'configured' -or $script:ProductionRootSpkiSha256 -eq 'UNCONFIGURED' -or
        $anchor.root_spki_sha256 -ne $script:ProductionRootSpkiSha256 -or
        (Get-Sha256Bytes $rootSpki) -ne $script:ProductionRootSpkiSha256) {
        Fail 'RETENTION_ROOT_PIN_MISMATCH'
    }
    return [pscustomobject]@{ Anchor = $anchor; RootSpki = $rootSpki }
}

function Test-DelegatedCertificate($RootContext, $Certificate, [string]$Purpose, [string]$Scope) {
    $anchor = $RootContext.Anchor
    if ($Certificate.schema_version -ne 'x4-delegated-purpose-certificate.v1' -or
        $Certificate.root_id -ne $anchor.root_id -or
        $Certificate.root_spki_sha256 -ne $script:ProductionRootSpkiSha256 -or
        $Certificate.purpose -ne $Purpose -or $Certificate.scope -ne $Scope -or
        [int]$Certificate.epoch -ne [int]$anchor.accepted_epochs.$Purpose -or
        $Certificate.algorithm -ne $script:SignatureAlgorithm) {
        Fail 'RETENTION_CERTIFICATE_POLICY_MISMATCH'
    }
    $policyDigest = Get-Sha256Bytes ([Text.Encoding]::UTF8.GetBytes("$Purpose|$($Certificate.epoch)|$Scope"))
    if ($Certificate.policy_digest -ne $policyDigest) { Fail 'RETENTION_CERTIFICATE_POLICY_MISMATCH' }
    $now = [DateTimeOffset]::UtcNow
    if ($now -lt [DateTimeOffset]::Parse([string]$Certificate.not_before) -or
        $now -ge [DateTimeOffset]::Parse([string]$Certificate.not_after)) { Fail 'RETENTION_CERTIFICATE_EXPIRED' }
    [byte[]]$delegatedSpki = [Convert]::FromBase64String([string]$Certificate.delegated_spki_der_base64)
    if ((Get-Sha256Bytes $delegatedSpki) -ne $Certificate.delegated_spki_sha256) {
        Fail 'RETENTION_DELEGATED_KEY_MISMATCH'
    }
    [byte[]]$rootSignature = [Convert]::FromBase64String([string]$Certificate.root_signature_base64)
    $root = [Security.Cryptography.ECDsa]::Create()
    try {
        $read = 0
        $root.ImportSubjectPublicKeyInfo($RootContext.RootSpki, [ref]$read)
        if (-not $root.VerifyData(
                (ConvertTo-CertificateBytes $Certificate), $rootSignature,
                [Security.Cryptography.HashAlgorithmName]::SHA256,
                [Security.Cryptography.DSASignatureFormat]::IeeeP1363FixedFieldConcatenation
            )) { Fail 'RETENTION_CERTIFICATE_SIGNATURE_INVALID' }
    }
    finally { $root.Dispose() }
    return $delegatedSpki
}

function Test-ContainedPath([string]$Path, [string]$Parent) {
    $full = [IO.Path]::GetFullPath($Path).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    $root = [IO.Path]::GetFullPath($Parent).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    return $full.StartsWith($root + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)
}

function Resolve-NoReparseDestination([string]$Path) {
    $full = [IO.Path]::GetFullPath($Path).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    $root = [IO.Path]::GetPathRoot($full)
    $relative = $full.Substring($root.Length).TrimStart([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    $segments = if ([string]::IsNullOrWhiteSpace($relative)) { @() } else { @($relative -split '[\\/]+' ) }
    $current = $root
    $existingCount = 0
    foreach ($segment in $segments) {
        $next = Join-Path $current $segment
        if (-not (Test-Path -LiteralPath $next)) { break }
        $item = Get-Item -LiteralPath $next -Force
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { Fail 'DESTINATION_REPARSE_POINT_REJECTED' }
        if (-not $item.PSIsContainer) { Fail 'DESTINATION_NOT_DIRECTORY' }
        $current = $item.FullName
        $existingCount += 1
    }
    $resolved = if (Test-Path -LiteralPath $current) { (Resolve-Path -LiteralPath $current).Path } else { $current }
    for ($index = $existingCount; $index -lt $segments.Count; $index += 1) {
        $resolved = Join-Path $resolved $segments[$index]
    }
    return [IO.Path]::GetFullPath($resolved).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
}

function Resolve-SafeDestination([string]$Path) {
    $full = Resolve-NoReparseDestination $Path
    $volume = [IO.Path]::GetPathRoot($full).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    if ([string]::IsNullOrWhiteSpace($full) -or $full.Equals($volume, [StringComparison]::OrdinalIgnoreCase)) {
        Fail 'FILESYSTEM_ROOT_REJECTED'
    }
    if ($full.Equals($repositoryRoot, [StringComparison]::OrdinalIgnoreCase) -or
        (Test-ContainedPath $full $repositoryRoot)) {
        Fail 'REPOSITORY_DESTINATION_REJECTED'
    }
    if ($full.Equals($publicPackageRoot, [StringComparison]::OrdinalIgnoreCase) -or
        (Test-ContainedPath $full $publicPackageRoot)) { Fail 'PUBLIC_PACKAGE_DESTINATION_REJECTED' }
    if ($full -match '(?i)[\\/]steamapps[\\/]common[\\/]X4 Foundations(?:[\\/]|$)') {
        Fail 'GAME_INSTALLATION_DESTINATION_REJECTED'
    }
    if ($full -match '(?i)[\\/]X4 Foundations[\\/]extensions(?:[\\/]|$)' -or
        $full -match '(?i)[\\/]extensions[\\/]live_galaxy(?:[\\/]|$)') {
        Fail 'PUBLIC_RUNTIME_DESTINATION_REJECTED'
    }
    if ($full -match '(?i)[\\/]Egosoft[\\/]X4[\\/][0-9]+[\\/]save(?:[\\/]|$)') {
        Fail 'GAME_SAVE_DESTINATION_REJECTED'
    }
    $segments = @($full.Split([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) | Where-Object { $_ })
    if ($segments.Count -gt 16) { Fail 'PATH_DEPTH_EXCEEDED' }
    return $full
}

function Set-OwnerOnly([string]$Path, [bool]$Directory) {
    if ($IsWindows) {
        $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User
        if ($Directory) {
            $security = [Security.AccessControl.DirectorySecurity]::new()
            $rule = [Security.AccessControl.FileSystemAccessRule]::new(
                $sid,
                [Security.AccessControl.FileSystemRights]::FullControl,
                [Security.AccessControl.InheritanceFlags]'ContainerInherit, ObjectInherit',
                [Security.AccessControl.PropagationFlags]::None,
                [Security.AccessControl.AccessControlType]::Allow
            )
        }
        else {
            $security = [Security.AccessControl.FileSecurity]::new()
            $rule = [Security.AccessControl.FileSystemAccessRule]::new(
                $sid,
                [Security.AccessControl.FileSystemRights]::FullControl,
                [Security.AccessControl.AccessControlType]::Allow
            )
        }
        $security.SetOwner($sid)
        $security.SetAccessRuleProtection($true, $false)
        [void]$security.AddAccessRule($rule)
        Set-Acl -LiteralPath $Path -AclObject $security
    }
    else {
        $mode = if ($Directory) {
            [IO.UnixFileMode]::UserRead -bor [IO.UnixFileMode]::UserWrite -bor [IO.UnixFileMode]::UserExecute
        }
        else {
            [IO.UnixFileMode]::UserRead -bor [IO.UnixFileMode]::UserWrite
        }
        [IO.File]::SetUnixFileMode($Path, $mode)
    }
}

function Assert-OwnerOnly([string]$Path, [bool]$Directory) {
    if ($IsWindows) {
        $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User
        $security = Get-Acl -LiteralPath $Path
        try {
            $ownerSid = ([Security.Principal.NTAccount]$security.Owner).Translate(
                [Security.Principal.SecurityIdentifier]
            ).Value
        }
        catch {
            $ownerSid = $security.Owner
        }
        if (-not $security.AreAccessRulesProtected -or $ownerSid -ne $sid.Value) { Fail 'PERMISSION_MISMATCH' }
        $rules = @($security.GetAccessRules($true, $true, [Security.Principal.SecurityIdentifier]))
        $allows = @($rules | Where-Object { $_.AccessControlType -eq [Security.AccessControl.AccessControlType]::Allow })
        if ($allows.Count -lt 1 -or @($allows | Where-Object { $_.IdentityReference.Value -ne $sid.Value }).Count -ne 0) {
            Fail 'PERMISSION_MISMATCH'
        }
        if (@($rules | Where-Object { $_.IsInherited }).Count -ne 0) { Fail 'PERMISSION_MISMATCH' }
    }
    else {
        $expected = if ($Directory) {
            [IO.UnixFileMode]::UserRead -bor [IO.UnixFileMode]::UserWrite -bor [IO.UnixFileMode]::UserExecute
        }
        else {
            [IO.UnixFileMode]::UserRead -bor [IO.UnixFileMode]::UserWrite
        }
        if ([IO.File]::GetUnixFileMode($Path) -ne $expected) { Fail 'PERMISSION_MISMATCH' }
    }
}

function Assert-ExactFields($Value, [string[]]$Expected) {
    $actual = if ($Value -is [Collections.IDictionary]) {
        @($Value.Keys | Sort-Object)
    }
    else {
        @($Value.PSObject.Properties.Name | Sort-Object)
    }
    $wanted = @($Expected | Sort-Object)
    if (($actual -join '|') -ne ($wanted -join '|')) { Fail 'UNEXPECTED_FIELD' }
}

function Read-Contracts {
    $runtime = Read-BoundedJson $runtimeContractPath 65536 'runtime-evidence.v1'
    $manifest = Read-BoundedJson $manifestContractPath 65536 'candidate-build-manifest-contract.v1'
    $sanitized = Read-BoundedJson $sanitizedContractPath 65536 'sanitized-ledger.v1'
    return [pscustomobject]@{
        Runtime = $runtime.Value
        Manifest = $manifest.Value
        Sanitized = $sanitized.Value
        RuntimeDigest = Get-Sha256Bytes $runtime.Bytes
    }
}

function Test-BuildManifest([string]$Path, $Contracts, [bool]$CheckGeneratedFiles) {
    $read = Read-BoundedJson $Path $Contracts.Sanitized.bounds.max_input_bytes $Contracts.Manifest.generated_schema_version
    $manifest = $read.Value
    foreach ($field in @($Contracts.Manifest.required_fields)) { $null = Require-Property $manifest $field }
    foreach ($field in @($Contracts.Manifest.required_digests)) { Require-Digest $manifest.$field }
    Require-Id $manifest.build_id
    Require-Id $manifest.group_id
    if ($manifest.developer_only -ne $true -or
        $manifest.execution_status -ne 'execution-ready-local-process' -or
        $manifest.native_execution_status -ne 'terminable-external-isolation' -or
        $manifest.local_readiness_verified -ne $true) { Fail 'INVALID_BUILD_STATUS' }
    $candidateIds = Require-Array $manifest.candidate_ids $Contracts.Manifest.bounds.max_candidates 1
    foreach ($id in $candidateIds) { Require-Id $id }
    if ((@($candidateIds | Sort-Object -Unique) -join '|') -ne (@($candidateIds) -join '|')) {
        Fail 'NON_DETERMINISTIC_CANDIDATES'
    }
    if ($manifest.runtime_evidence_schema_digest -ne $Contracts.RuntimeDigest) { Fail 'STALE_RUNTIME_SCHEMA_DIGEST' }
    foreach ($source in @(
        @{ Name = 'dossier_digest'; Path = $dossierPath },
        @{ Name = 'registry_digest'; Path = $registryPath },
        @{ Name = 'coverage_digest'; Path = $coveragePath },
        @{ Name = 'matrix_digest'; Path = $matrixPath }
    )) {
        if ($manifest.($source.Name) -ne (Get-Sha256File $source.Path)) { Fail 'STALE_IDENTITY_DIGEST' }
    }
    $package = Require-Property $manifest 'package_conformance'
    $packageBytes = [Text.Encoding]::UTF8.GetBytes(($package | ConvertTo-Json -Compress -Depth 32))
    if ((Get-Sha256Bytes $packageBytes) -ne $manifest.package_conformance_digest) {
        Fail 'PACKAGE_CONFORMANCE_DIGEST_MISMATCH'
    }
    if ($package.verdict -ne 'conformant' -or $package.classification -ne 'local-only' -or
        $package.evidence_level -ne 'packaged-static' -or $package.dossier_digest -ne $manifest.dossier_digest -or
        $package.coverage_digest -ne $manifest.coverage_digest) {
        Fail 'INVALID_PACKAGE_CONFORMANCE'
    }
    $files = Require-Array $manifest.generated_files $Contracts.Manifest.bounds.max_generated_files 1
    $paths = @($files | ForEach-Object { [string](Require-Property $_ 'path') })
    if (($paths -join '|') -ne (@($Contracts.Manifest.required_generated_files) -join '|')) {
        Fail 'INVALID_GENERATED_FILE_SET'
    }
    [long]$totalBytes = 0
    $groupRoot = Split-Path -Parent (Split-Path -Parent ([IO.Path]::GetFullPath($Path)))
    foreach ($file in $files) {
        Require-Text $file.path 256
        Require-Integer $file.bytes $Contracts.Manifest.bounds.max_generated_file_bytes
        Require-Digest $file.sha256
        $totalBytes += [long]$file.bytes
        if ($CheckGeneratedFiles) {
            $physical = [IO.Path]::GetFullPath((Join-Path $groupRoot $file.path))
            if (-not (Test-ContainedPath $physical $groupRoot) -or -not (Test-Path -LiteralPath $physical -PathType Leaf)) {
                Fail 'GENERATED_FILE_MISSING'
            }
            if ((Get-Item -LiteralPath $physical).Length -ne [long]$file.bytes -or (Get-Sha256File $physical) -ne $file.sha256) {
                Fail 'GENERATED_FILE_DIGEST_MISMATCH'
            }
        }
    }
    if ($totalBytes -gt $Contracts.Manifest.bounds.max_generated_total_bytes) { Fail 'BOUND_EXCEEDED' }
    if ($CheckGeneratedFiles) {
        $generatedContractPath = Join-Path $groupRoot 'manifest/package-conformance.v1.json'
        $liveOutput = @(& pwsh -NoProfile -File $packageConformancePath `
            -PackageRoot $groupRoot -ContractPath $generatedContractPath `
            -DossierPath $dossierPath -CoveragePath $coveragePath 2>&1)
        if ($LASTEXITCODE -ne 0 -or $liveOutput.Count -ne 1) { Fail 'PACKAGE_CONFORMANCE_LIVE_MISMATCH' }
        try { $live = $liveOutput[0].ToString() | ConvertFrom-Json -Depth 32 -DateKind String }
        catch { Fail 'PACKAGE_CONFORMANCE_LIVE_MISMATCH' }
        $livePackage = [ordered]@{
            schema_version = $live.schema_version
            verdict = $live.verdict
            classification = $live.classification
            evidence_level = $live.evidence_level
            graph_digest = $live.graph_digest
            dossier_digest = $live.dossier_digest
            coverage_digest = $live.coverage_digest
        }
        $liveBytes = [Text.Encoding]::UTF8.GetBytes(($livePackage | ConvertTo-Json -Compress -Depth 32))
        if ($live.verdict -ne 'conformant' -or $live.classification -ne 'local-only' -or
            (Get-Sha256Bytes $liveBytes) -ne $manifest.package_conformance_digest -or
            ($livePackage | ConvertTo-Json -Compress -Depth 32) -cne
                ($package | ConvertTo-Json -Compress -Depth 32)) {
            Fail 'PACKAGE_CONFORMANCE_LIVE_MISMATCH'
        }
        $componentBindings = [ordered]@{
            dispatcher_digest = (Join-Path $PSScriptRoot 'run-candidate-package.ps1')
            adapter_digest = (Join-Path $groupRoot 'tools/x4-verification/candidate-adapters.psm1')
            attestation_module_digest = (Join-Path $groupRoot 'tools/x4-verification/producer-attestation.psm1')
            worker_digest = (Join-Path $groupRoot 'tools/x4-verification/isolation/candidate-worker.ps1')
            launcher_digest = (Join-Path $groupRoot 'tools/x4-verification/isolation/invoke-candidate-worker.ps1')
            worker_protocol_digest = (Join-Path $groupRoot 'tools/x4-verification/contracts/candidate-worker-protocol.v1.json')
            runtime_evidence_schema_digest = (Join-Path $groupRoot 'tools/x4-verification/contracts/runtime-evidence.v1.json')
            owner_root_anchor_digest = (Join-Path $groupRoot 'tools/x4-verification/contracts/owner-root-anchor.v1.json')
        }
        foreach ($binding in $componentBindings.GetEnumerator()) {
            if ($manifest.([string]$binding.Key) -ne (Get-Sha256File ([string]$binding.Value))) {
                Fail 'GENERATED_FILE_DIGEST_MISMATCH'
            }
        }
    }
    return [pscustomobject]@{ Value = $manifest; Bytes = $read.Bytes; CandidateIds = @($candidateIds) }
}

function Test-LocalEvidenceStream([string]$Path, $Build, $Contracts) {
    $bytes = Read-BoundedBytes $Path $Contracts.Runtime.bounds.max_total_bytes
    if ($bytes.Length -eq 0 -or $bytes[-1] -ne 10) { Fail 'INTERRUPTED_JSONL' }
    $lines = @([Text.Encoding]::UTF8.GetString($bytes).TrimEnd("`n").Split("`n"))
    if ($lines.Count -lt 1 -or $lines.Count -gt $Contracts.Manifest.bounds.max_candidates) { Fail 'BOUND_EXCEEDED' }
    $candidates = [Collections.Generic.List[object]]::new()
    $ids = [Collections.Generic.List[string]]::new()
    $runId = $null
    foreach ($line in $lines) {
        if ([Text.Encoding]::UTF8.GetByteCount($line) -gt $Contracts.Runtime.bounds.max_row_bytes) { Fail 'BOUND_EXCEEDED' }
        try { $row = $line | ConvertFrom-Json -Depth 16 -DateKind String }
        catch { Fail 'MALFORMED_JSONL' }
        Assert-ExactFields $row @(
            'schema_version', 'run_id', 'candidate_id', 'build_id', 'group_id',
            'execution_verdict', 'contract_verdict', 'effect_verdict',
            'completeness', 'diagnostic_code', 'evidence_classification'
        )
        foreach ($idName in @('run_id', 'candidate_id', 'build_id', 'group_id')) { Require-Id $row.$idName }
        if ($row.schema_version -ne 'candidate-local-evidence.v1' -or
            $row.evidence_classification -ne 'authenticated-local-contract' -or
            $row.build_id -ne $Build.Value.build_id -or $row.group_id -ne $Build.Value.group_id -or
            $row.execution_verdict -ne 'pass' -or $row.contract_verdict -ne 'pass' -or
            $row.effect_verdict -ne 'pass' -or $row.completeness -ne 'complete') {
            Fail 'INVALID_LOCAL_EVIDENCE'
        }
        if ($null -eq $runId) { $runId = $row.run_id }
        elseif ($runId -ne $row.run_id) { Fail 'EVIDENCE_IDENTITY_MISMATCH' }
        $ids.Add([string]$row.candidate_id)
        $candidates.Add([ordered]@{
            candidate_id = [string]$row.candidate_id
            execution_verdict = 'pass'
            contract_verdict = 'pass'
            effect_verdict = 'pass'
            disposition = 'retain'
        })
    }
    if (($ids -join '|') -ne (@($Build.CandidateIds) -join '|')) { Fail 'BUILD_EVIDENCE_IDENTITY_MISMATCH' }
    return [pscustomobject]@{
        Bytes = $bytes; RunId = $runId; PriorDossierId = 'phase-05.1-dossier'
        Candidates = @($candidates); CandidateIds = @($ids)
    }
}

function Test-EvidenceStream([string]$Path, $Build, $Contracts) {
    $bytes = Read-BoundedBytes $Path $Contracts.Runtime.bounds.max_total_bytes
    if ($bytes.Length -eq 0 -or $bytes[-1] -ne 10) { Fail 'INTERRUPTED_JSONL' }
    $text = [Text.Encoding]::UTF8.GetString($bytes)
    $lines = @($text.TrimEnd("`n").Split("`n"))
    if ($lines.Count -gt $Contracts.Runtime.bounds.max_output_rows -or $lines.Count -lt 1) { Fail 'BOUND_EXCEEDED' }
    $rows = [Collections.Generic.List[object]]::new()
    foreach ($line in $lines) {
        if ($line.EndsWith("`r") -or [Text.Encoding]::UTF8.GetByteCount($line) -gt $Contracts.Runtime.bounds.max_row_bytes) {
            Fail 'INVALID_JSONL_LINE'
        }
        try { $row = $line | ConvertFrom-Json }
        catch { Fail 'MALFORMED_JSONL' }
        Assert-ExactFields $row @(@($Contracts.Runtime.required_fields + 'evidence_classification') | Sort-Object -Unique)
        foreach ($field in @($Contracts.Runtime.required_fields)) { $null = Require-Property $row $field }
        if ($row.schema_version -ne $Contracts.Runtime.schema_version -or
            $row.evidence_classification -ne $Contracts.Runtime.evidence_classification -or
            $row.digest_algorithm -ne $Contracts.Runtime.digest_algorithm) {
            Fail 'UNSUPPORTED_EVIDENCE_SCHEMA'
        }
        foreach ($field in @('game_version', 'scenario_id', 'candidate_source', 'expected_result', 'actual_result')) {
            Require-Text $row.$field $Contracts.Runtime.bounds.max_string_bytes
        }
        foreach ($field in @('run_id', 'candidate_id', 'prior_dossier_id', 'build_id')) { Require-Id $row.$field }
        foreach ($field in @('prior_dossier_digest', 'build_profile_digest', 'record_digest')) { Require-Digest $row.$field }
        $mods = Require-Array $row.mod_list $Contracts.Runtime.bounds.max_mods
        foreach ($mod in $mods) { Require-Text $mod $Contracts.Runtime.bounds.max_string_bytes }
        if ((@($mods | Sort-Object -Unique) -join '|') -ne (@($mods) -join '|')) { Fail 'NON_DETERMINISTIC_MOD_LIST' }
        Require-Integer $row.elapsed_real_ms
        Require-Integer $row.elapsed_game_ms
        Require-Integer $row.work_units $Contracts.Runtime.bounds.max_work_units_per_step
        Require-Integer $row.observation_count $Contracts.Runtime.bounds.max_observations_per_step
        if ($row.stage_id -notin @($Contracts.Runtime.stage_order) -or
            $row.execution_verdict -notin @($Contracts.Runtime.verdicts.execution) -or
            $row.contract_verdict -notin @($Contracts.Runtime.verdicts.contract) -or
            $row.effect_verdict -notin @($Contracts.Runtime.verdicts.effect) -or
            $row.completeness -notin @($Contracts.Runtime.completeness) -or
            $row.seta_state -notin @($Contracts.Runtime.seta_states) -or
            $row.failure_point -notin @($Contracts.Runtime.failure_points) -or
            $row.failure_reason -notin @($Contracts.Runtime.failure_reasons)) {
            Fail 'INVALID_EVIDENCE_ENUM'
        }
        if (($row.failure_point -eq 'none') -ne ($row.failure_reason -eq 'none')) { Fail 'INVALID_FAILURE_IDENTITY' }
        $canonicalBytes = [Text.Encoding]::UTF8.GetBytes($row.canonical_digest_payload)
        if ((Get-Sha256Bytes $canonicalBytes) -ne $row.record_digest) { Fail 'EVIDENCE_DIGEST_MISMATCH' }
        try { $canonicalValue = $row.canonical_digest_payload | ConvertFrom-Json }
        catch { Fail 'INVALID_CANONICAL_PAYLOAD' }
        if ((ConvertTo-CanonicalJson $canonicalValue) -ne $row.canonical_digest_payload) { Fail 'NON_CANONICAL_PAYLOAD' }
        $payload = [ordered]@{}
        foreach ($property in @($row.PSObject.Properties | Sort-Object Name)) {
            if ($property.Name -notin @('digest_algorithm', 'canonical_digest_payload', 'record_digest')) {
                $payload[$property.Name] = $property.Value
            }
        }
        if ((ConvertTo-CanonicalJson $payload) -ne $row.canonical_digest_payload) { Fail 'CANONICAL_PAYLOAD_MISMATCH' }
        $rows.Add($row)
    }
    if ($rows.Count % 3 -ne 0) { Fail 'INCOMPLETE_CANDIDATE_ROWS' }
    $candidateIds = [Collections.Generic.List[string]]::new()
    $candidates = [Collections.Generic.List[object]]::new()
    $first = $rows[0]
    for ($index = 0; $index -lt $rows.Count; $index += 3) {
        $group = @($rows[$index], $rows[$index + 1], $rows[$index + 2])
        $candidateId = $group[0].candidate_id
        $candidateIds.Add($candidateId)
        for ($stageIndex = 0; $stageIndex -lt 3; $stageIndex++) {
            $row = $group[$stageIndex]
            if ($row.candidate_id -ne $candidateId -or $row.stage_id -ne $Contracts.Runtime.stage_order[$stageIndex]) {
                Fail 'INVALID_STAGE_ORDER'
            }
            foreach ($identity in @('run_id', 'build_id', 'build_profile_digest', 'prior_dossier_id', 'prior_dossier_digest', 'game_version', 'scenario_id', 'candidate_source', 'expected_result')) {
                if ((ConvertTo-CanonicalJson $row.$identity) -ne (ConvertTo-CanonicalJson $group[0].$identity)) {
                    Fail 'EVIDENCE_IDENTITY_MISMATCH'
                }
            }
            if ((ConvertTo-CanonicalJson $row.mod_list) -ne (ConvertTo-CanonicalJson $group[0].mod_list)) {
                Fail 'EVIDENCE_IDENTITY_MISMATCH'
            }
        }
        if ($group[0].contract_verdict -ne 'not_run' -or $group[0].effect_verdict -ne 'not_run' -or
            $group[1].effect_verdict -ne 'not_run') {
            Fail 'INVALID_VERDICT_PROGRESSION'
        }
        if ($group[1].execution_verdict -ne $group[0].execution_verdict -or
            $group[2].execution_verdict -ne $group[0].execution_verdict -or
            $group[2].contract_verdict -ne $group[1].contract_verdict) {
            Fail 'NON_MONOTONIC_VERDICT'
        }
        if ($group[0].execution_verdict -ne 'pass') {
            if ($group[1].contract_verdict -ne 'not_run' -or $group[2].contract_verdict -ne 'not_run' -or
                $group[2].effect_verdict -ne 'not_run') { Fail 'FAILED_STAGE_CONTINUED' }
        }
        elseif ($group[1].contract_verdict -ne 'pass' -and $group[2].effect_verdict -ne 'not_run') {
            Fail 'FAILED_STAGE_CONTINUED'
        }
        $failureRows = @($group | Where-Object { $_.failure_point -ne 'none' })
        if ($failureRows.Count -gt 0) {
            $firstFailure = $failureRows[0]
            foreach ($later in @($group | Where-Object { [array]::IndexOf($group, $_) -ge [array]::IndexOf($group, $firstFailure) })) {
                if ($later.failure_point -ne $firstFailure.failure_point -or
                    $later.failure_reason -ne $firstFailure.failure_reason) {
                    Fail 'FAILURE_IDENTITY_CHANGED'
                }
            }
        }
        if ($group[2].effect_verdict -eq 'pass' -and $group[2].actual_result -ne $group[2].expected_result) {
            Fail 'UNEXPECTED_EFFECT_PASS'
        }
        $candidates.Add([ordered]@{
            candidate_id = $candidateId
            execution_verdict = $group[2].execution_verdict
            contract_verdict = $group[2].contract_verdict
            effect_verdict = $group[2].effect_verdict
            disposition = 'retain'
        })
    }
    if (($candidateIds -join '|') -ne (@($Build.CandidateIds) -join '|')) { Fail 'BUILD_EVIDENCE_IDENTITY_MISMATCH' }
    if ($first.build_id -ne $Build.Value.build_id -or
        $first.build_profile_digest -ne $Build.Value.build_profile_digest -or
        $first.prior_dossier_digest -ne $Build.Value.dossier_digest) {
        Fail 'BUILD_EVIDENCE_IDENTITY_MISMATCH'
    }
    return [pscustomobject]@{
        Bytes = $bytes
        RunId = $first.run_id
        PriorDossierId = $first.prior_dossier_id
        Candidates = @($candidates)
    }
}

function Test-ProducerAttestation([string]$EvidencePath, $Build, $Evidence, [string]$EvidenceDigest, $RootContext, [string]$AttestationPath = '') {
    $attestationPath = if ([string]::IsNullOrWhiteSpace($AttestationPath)) { "$EvidencePath.attestation.json" } else { $AttestationPath }
    if (-not (Test-Path -LiteralPath $attestationPath -PathType Leaf)) { Fail 'RETENTION_PRODUCER_ATTESTATION_MISSING' }
    $read = Read-BoundedJson $attestationPath 65536 'candidate-producer-attestation.v1'
    $envelope = $read.Value
    try { $null = producer-attestation\Test-CandidateProducerEnvelopeFields $envelope }
    catch { Fail ([string]$_.Exception.Message) }
    $certificate = $envelope.certificate
    [byte[]]$delegatedSpki = Test-DelegatedCertificate $RootContext $certificate $script:ProducerPurpose $script:ProducerScope
    $payload = $envelope.payload
    try {
        $null = producer-attestation\Test-CandidateProducerPayload -Payload $payload `
            -CertificateId ([string]$certificate.certificate_id) `
            -Epoch ([int]$RootContext.Anchor.accepted_epochs.'candidate-producer') `
            -Scope $script:ProducerScope -Now ([DateTimeOffset]::UtcNow)
    }
    catch { Fail ([string]$_.Exception.Message) }
    $payloadBytes = [Text.UTF8Encoding]::new($false).GetBytes(
        (producer-attestation\ConvertTo-CanonicalJson $payload)
    )
    if ($envelope.payload_digest -ne (Get-Sha256Bytes $payloadBytes) -or
        $payload.classification -ne 'authenticated-local-contract' -or
        $payload.build_id -ne $Build.Value.build_id -or $payload.run_id -ne $Evidence.RunId -or
        $payload.evidence_digest -ne $EvidenceDigest -or
        (@($payload.candidate_ids) -join '|') -ne (@($Evidence.CandidateIds) -join '|')) {
        Fail 'RETENTION_PRODUCER_IDENTITY_MISMATCH'
    }
    foreach ($binding in @(
        'dispatcher_digest', 'adapter_digest', 'attestation_module_digest', 'worker_digest', 'launcher_digest',
        'worker_protocol_digest', 'runtime_evidence_schema_digest',
        'package_conformance_digest', 'matrix_digest'
    )) {
        if ($payload.$binding -ne $Build.Value.$binding) { Fail 'RETENTION_PRODUCER_IDENTITY_MISMATCH' }
    }
    [byte[]]$signature = [Convert]::FromBase64String([string]$envelope.signature_base64)
    $verifier = [Security.Cryptography.ECDsa]::Create()
    try {
        $readCount = 0
        $verifier.ImportSubjectPublicKeyInfo($delegatedSpki, [ref]$readCount)
        if (-not $verifier.VerifyData(
                $payloadBytes, $signature, [Security.Cryptography.HashAlgorithmName]::SHA256,
                [Security.Cryptography.DSASignatureFormat]::IeeeP1363FixedFieldConcatenation
            )) { Fail 'RETENTION_PRODUCER_SIGNATURE_INVALID' }
    }
    finally { $verifier.Dispose() }
    return [pscustomobject]@{ Bytes = $read.Bytes; Digest = Get-Sha256Bytes $read.Bytes }
}

function Get-LocatorSigningAuthority($RootContext) {
    if ($script:TestOnlyHarness) {
        if ([string]::IsNullOrWhiteSpace($script:TestAuthorityPath)) { Fail 'TEST_AUTHORITY_PATH_REJECTED' }
        $authorityPath = [IO.Path]::GetFullPath($script:TestAuthorityPath)
        $authority = (Read-BoundedJson $authorityPath 65536 'retention-test-authority.v1').Value
        $certificate = $authority.locator_certificate
        $null = Test-DelegatedCertificate $RootContext $certificate $script:LocatorPurpose $script:LocatorScope
        [byte[]]$privateBytes = [Convert]::FromBase64String([string]$authority.locator_private_pkcs8_base64)
        $signer = [Security.Cryptography.ECDsa]::Create()
        $read = 0
        $signer.ImportPkcs8PrivateKey($privateBytes, [ref]$read)
        return [pscustomobject]@{ Certificate = $certificate; Signer = $signer; Key = $null }
    }
    $certificatePath = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'LiveGalaxy/authority/retention-locator-certificate.v1.json'
    if (-not (Test-Path -LiteralPath $certificatePath -PathType Leaf)) { Fail 'RETENTION_ATTESTATION_UNCONFIGURED' }
    $certificate = (Read-BoundedJson $certificatePath 32768 'x4-delegated-purpose-certificate.v1').Value
    $null = Test-DelegatedCertificate $RootContext $certificate $script:LocatorPurpose $script:LocatorScope
    try {
        $key = [Security.Cryptography.CngKey]::Open(
            [string]$certificate.windows_key_name,
            [Security.Cryptography.CngProvider]::MicrosoftSoftwareKeyStorageProvider,
            [Security.Cryptography.CngKeyOpenOptions]::UserKey
        )
        if (-not (producer-attestation\Test-CngSigningKeyPolicy $key)) {
            $key.Dispose(); Fail 'RETENTION_DELEGATED_KEY_USER_PRESENCE_REQUIRED'
        }
        $signer = [Security.Cryptography.ECDsaCng]::new($key)
        if ((Get-Sha256Bytes $signer.ExportSubjectPublicKeyInfo()) -ne $certificate.delegated_spki_sha256) {
            $signer.Dispose(); $key.Dispose(); Fail 'RETENTION_DELEGATED_KEY_MISMATCH'
        }
        return [pscustomobject]@{ Certificate = $certificate; Signer = $signer; Key = $key }
    }
    catch [Security.Cryptography.CryptographicException] { Fail 'RETENTION_ATTESTATION_UNCONFIGURED' }
}

function New-SignedLocator($Payload, $Authority) {
    $payloadBytes = [Text.UTF8Encoding]::new($false).GetBytes((ConvertTo-CanonicalJson $Payload))
    [byte[]]$signature = $Authority.Signer.SignData(
        $payloadBytes, [Security.Cryptography.HashAlgorithmName]::SHA256,
        [Security.Cryptography.DSASignatureFormat]::IeeeP1363FixedFieldConcatenation
    )
    $locator = [ordered]@{}
    foreach ($property in $Payload.GetEnumerator()) { $locator[$property.Key] = $property.Value }
    $locator.delegation_certificate = $Authority.Certificate
    $locator.payload_digest = Get-Sha256Bytes $payloadBytes
    $locator.signature_base64 = [Convert]::ToBase64String($signature)
    return $locator
}

function Test-LocatorSignature($Locator, $RootContext) {
    $certificate = $Locator.delegation_certificate
    [byte[]]$delegatedSpki = Test-DelegatedCertificate $RootContext $certificate $script:LocatorPurpose $script:LocatorScope
    $payload = [ordered]@{}
    foreach ($property in $Locator.PSObject.Properties) {
        if ($property.Name -notin @('delegation_certificate', 'payload_digest', 'signature_base64')) {
            $payload[$property.Name] = $property.Value
        }
    }
    $payloadBytes = [Text.UTF8Encoding]::new($false).GetBytes((ConvertTo-CanonicalJson $payload))
    if ($Locator.payload_digest -ne (Get-Sha256Bytes $payloadBytes)) { Fail 'LOCATOR_SIGNATURE_INVALID' }
    [byte[]]$signature = [Convert]::FromBase64String([string]$Locator.signature_base64)
    $verifier = [Security.Cryptography.ECDsa]::Create()
    try {
        $read = 0
        $verifier.ImportSubjectPublicKeyInfo($delegatedSpki, [ref]$read)
        if (-not $verifier.VerifyData(
                $payloadBytes, $signature, [Security.Cryptography.HashAlgorithmName]::SHA256,
                [Security.Cryptography.DSASignatureFormat]::IeeeP1363FixedFieldConcatenation
            )) { Fail 'LOCATOR_SIGNATURE_INVALID' }
    }
    finally { $verifier.Dispose() }
}

function New-SanitizedBase($Build, $Evidence, $Contracts, [string]$EvidenceDigest, [string]$ManifestDigest, [string]$ProducerDigest) {
    $runPayload = "$($Evidence.RunId)|$EvidenceDigest|$ManifestDigest"
    return [ordered]@{
        schema_version = $Contracts.Sanitized.schema_version
        ledger_id = "retained-$($Evidence.RunId)"
        run_id = $Evidence.RunId
        build_id = $Build.Value.build_id
        group_id = $Build.Value.group_id
        evidence_classification = 'authenticated-local-contract'
        runtime_evidence_schema_version = $Contracts.Runtime.schema_version
        build_manifest_schema_version = $Contracts.Manifest.generated_schema_version
        verdict = $Contracts.Sanitized.verdict
        retention_disposition = 'retained'
        identity_digests = [ordered]@{
            dossier_digest = $Build.Value.dossier_digest
            registry_digest = $Build.Value.registry_digest
            coverage_digest = $Build.Value.coverage_digest
            matrix_digest = $Build.Value.matrix_digest
            build_profile_digest = $Build.Value.build_profile_digest
            package_conformance_digest = $Build.Value.package_conformance_digest
            runtime_evidence_schema_digest = $Build.Value.runtime_evidence_schema_digest
            build_manifest_digest = $ManifestDigest
            evidence_digest = $EvidenceDigest
            producer_attestation_digest = $ProducerDigest
            run_digest = Get-Sha256Bytes ([Text.Encoding]::UTF8.GetBytes($runPayload))
        }
        candidates = @($Evidence.Candidates)
    }
}

function Test-SanitizedBase($Value, $Contracts, [bool]$LocatorDigestExpected) {
    $script:diagnosticId = 'sanitized-top-fields'
    Assert-ExactFields $Value @($Contracts.Sanitized.required_fields)
    if ($Value.schema_version -ne $Contracts.Sanitized.schema_version -or $Value.verdict -ne 'retained' -or
        $Value.retention_disposition -notin @($Contracts.Sanitized.retention_dispositions) -or
        $Value.evidence_classification -notin @($Contracts.Sanitized.evidence_classifications)) {
        Fail 'INVALID_SANITIZED_LEDGER'
    }
    foreach ($id in @('ledger_id', 'run_id', 'build_id', 'group_id')) { Require-Id $Value.$id }
    $expectedDigests = @($Contracts.Sanitized.required_identity_digests)
    if (-not $LocatorDigestExpected) { $expectedDigests = @($expectedDigests | Where-Object { $_ -ne 'locator_digest' }) }
    $script:diagnosticId = 'sanitized-digest-fields'
    Assert-ExactFields $Value.identity_digests $expectedDigests
    foreach ($name in $expectedDigests) { Require-Digest $Value.identity_digests.$name }
    $candidates = Require-Array $Value.candidates $Contracts.Sanitized.bounds.max_candidates 1
    foreach ($candidate in $candidates) {
        $script:diagnosticId = 'sanitized-candidate-fields'
        Assert-ExactFields $candidate @($Contracts.Sanitized.candidate_required_fields)
        Require-Id $candidate.candidate_id
        if ($candidate.execution_verdict -notin @($Contracts.Sanitized.execution_verdicts) -or
            $candidate.contract_verdict -notin @($Contracts.Sanitized.contract_verdicts) -or
            $candidate.effect_verdict -notin @($Contracts.Sanitized.effect_verdicts) -or
            $candidate.disposition -notin @($Contracts.Sanitized.candidate_dispositions)) {
            Fail 'INVALID_SANITIZED_LEDGER'
        }
    }
}

function Write-PrivateBytes([string]$Path, [byte[]]$Bytes) {
    [IO.File]::WriteAllBytes($Path, $Bytes)
    Set-OwnerOnly $Path $false
    Assert-OwnerOnly $Path $false
}

function Read-VerifiedLocator([string]$Path, $Contracts) {
    $rootContext = Get-RootContext
    $locatorRead = Read-BoundedJson $Path $Contracts.Sanitized.bounds.max_locator_bytes $Contracts.Sanitized.private_locator_schema_version
    $locator = $locatorRead.Value
    Test-LocatorSignature $locator $rootContext
    Assert-OwnerOnly (Split-Path -Parent ([IO.Path]::GetFullPath($Path))) $true
    Assert-OwnerOnly ([IO.Path]::GetFullPath($Path)) $false
    foreach ($field in @(
        'logical_artifact_id', 'run_id', 'evidence_path', 'build_manifest_path',
        'producer_attestation_path', 'evidence_digest', 'build_manifest_digest',
        'producer_attestation_digest', 'sanitized', 'delegation_certificate',
        'payload_digest', 'signature_base64'
    )) {
        $null = Require-Property $locator $field
    }
    Require-Id $locator.logical_artifact_id
    Require-Id $locator.run_id
    Require-Digest $locator.evidence_digest
    Require-Digest $locator.build_manifest_digest
    Require-Digest $locator.producer_attestation_digest
    $locatorDirectory = Split-Path -Parent ([IO.Path]::GetFullPath($Path))
    $retainedEvidence = [IO.Path]::GetFullPath($locator.evidence_path)
    $retainedManifest = [IO.Path]::GetFullPath($locator.build_manifest_path)
    $retainedProducer = [IO.Path]::GetFullPath($locator.producer_attestation_path)
    if ((Split-Path -Parent $retainedEvidence) -ne $locatorDirectory -or
        (Split-Path -Parent $retainedManifest) -ne $locatorDirectory -or
        (Split-Path -Parent $retainedProducer) -ne $locatorDirectory -or
        (Split-Path -Leaf $retainedEvidence) -ne 'runtime-evidence.v1.jsonl' -or
        (Split-Path -Leaf $retainedManifest) -ne 'build-manifest.v1.json' -or
        (Split-Path -Leaf $retainedProducer) -ne 'producer-attestation.v1.json') {
        Fail 'LOCATOR_PATH_MISMATCH'
    }
    Assert-OwnerOnly $retainedEvidence $false
    Assert-OwnerOnly $retainedManifest $false
    Assert-OwnerOnly $retainedProducer $false
    if ((Get-Sha256File $retainedEvidence) -ne $locator.evidence_digest -or
        (Get-Sha256File $retainedManifest) -ne $locator.build_manifest_digest -or
        (Get-Sha256File $retainedProducer) -ne $locator.producer_attestation_digest) {
        Fail 'RETAINED_DIGEST_MISMATCH'
    }
    $build = Test-BuildManifest $retainedManifest $Contracts $false
    $evidence = Test-LocalEvidenceStream $retainedEvidence $build $Contracts
    $producer = Test-ProducerAttestation $retainedEvidence $build $evidence $locator.evidence_digest $rootContext $retainedProducer
    if ($producer.Digest -ne $locator.producer_attestation_digest) { Fail 'RETAINED_DIGEST_MISMATCH' }
    $base = New-SanitizedBase $build $evidence $Contracts $locator.evidence_digest $locator.build_manifest_digest $producer.Digest
    Test-SanitizedBase $locator.sanitized $Contracts $false
    if ((ConvertTo-CanonicalJson $base) -ne (ConvertTo-CanonicalJson $locator.sanitized)) {
        Fail 'LOCATOR_IDENTITY_MISMATCH'
    }
    $base.identity_digests.locator_digest = Get-Sha256Bytes $locatorRead.Bytes
    Test-SanitizedBase $base $Contracts $true
    return $base
}

function Remove-ExactCleanupTargets {
    # Never recursively remove a path after a separate containment/reparse
    # check. A failed private staging directory is retained for explicit,
    # identity-aware owner cleanup rather than following a swapped junction.
}

function Write-Rejection {
    [ordered]@{
        schema_version = 'evidence-retention-result.v1'
        verdict = 'rejected'
        reason_code = $script:failureCode
        diagnostic_id = $script:diagnosticId
    } | ConvertTo-Json -Compress
}

try {
    if ($script:SimulateUnsupportedPlatform -or (-not $IsWindows -and -not $script:TestOnlyHarness)) {
        Fail 'RETENTION_ATTESTATION_PLATFORM_UNSUPPORTED'
    }
    $script:diagnosticId = 'contracts'
    $contracts = Read-Contracts
    if ($PSCmdlet.ParameterSetName -eq 'Verify') {
        $script:diagnosticId = 'retained-reread'
        $sanitized = Read-VerifiedLocator ([IO.Path]::GetFullPath($VerifyLocatorPath)) $contracts
        Write-Output ($sanitized | ConvertTo-Json -Compress -Depth 32)
        exit 0
    }

    $script:diagnosticId = 'build-manifest'
    $build = Test-BuildManifest ([IO.Path]::GetFullPath($BuildManifestPath)) $contracts $true
    $script:diagnosticId = 'runtime-evidence'
    $evidence = Test-LocalEvidenceStream ([IO.Path]::GetFullPath($EvidencePath)) $build $contracts
    $evidenceDigest = Get-Sha256Bytes $evidence.Bytes
    $manifestDigest = Get-Sha256Bytes $build.Bytes
    $script:diagnosticId = 'producer-attestation'
    $rootContext = Get-RootContext
    $producer = Test-ProducerAttestation ([IO.Path]::GetFullPath($EvidencePath)) $build $evidence $evidenceDigest $rootContext
    $script:diagnosticId = 'locator-authority'
    $locatorAuthority = Get-LocatorSigningAuthority $rootContext
    $sanitizedBase = New-SanitizedBase $build $evidence $contracts $evidenceDigest $manifestDigest $producer.Digest
    Test-SanitizedBase $sanitizedBase $contracts $false

    $script:diagnosticId = 'destination'
    $destination = Resolve-SafeDestination $DestinationRoot
    $script:cleanupDestination = $destination
    if (Test-Path -LiteralPath $destination) {
        if (-not (Test-Path -LiteralPath $destination -PathType Container)) { Fail 'DESTINATION_NOT_DIRECTORY' }
        Assert-OwnerOnly $destination $true
    }
    else {
        $null = New-Item -ItemType Directory -Path $destination -Force
        $script:createdDestination = $true
        Set-OwnerOnly $destination $true
        Assert-OwnerOnly $destination $true
    }
    $finalRoot = Join-Path $destination $evidence.RunId
    if (-not (Test-ContainedPath $finalRoot $destination) -or (Test-Path -LiteralPath $finalRoot)) {
        Fail 'RUN_DESTINATION_EXISTS'
    }
    $stagingRoot = Join-Path $destination (".{0}.{1}.partial" -f $evidence.RunId, [guid]::NewGuid().ToString('N'))
    if (-not (Test-ContainedPath $stagingRoot $destination)) { Fail 'STAGING_PATH_ESCAPE' }
    $script:cleanupRoot = $stagingRoot
    $null = New-Item -ItemType Directory -Path $stagingRoot
    Set-OwnerOnly $stagingRoot $true
    Assert-OwnerOnly $stagingRoot $true

    $retainedEvidence = Join-Path $stagingRoot 'runtime-evidence.v1.jsonl'
    $retainedManifest = Join-Path $stagingRoot 'build-manifest.v1.json'
    $retainedProducer = Join-Path $stagingRoot 'producer-attestation.v1.json'
    Write-PrivateBytes $retainedEvidence $evidence.Bytes
    Write-PrivateBytes $retainedManifest $build.Bytes
    Write-PrivateBytes $retainedProducer $producer.Bytes
    if ((Get-Sha256File $retainedEvidence) -ne $evidenceDigest -or (Get-Sha256File $retainedManifest) -ne $manifestDigest) {
        Fail 'RETAINED_DIGEST_MISMATCH'
    }
    if ((Get-Sha256File $retainedProducer) -ne $producer.Digest) { Fail 'RETAINED_DIGEST_MISMATCH' }
    $locatorPayload = [ordered]@{
        schema_version = $contracts.Sanitized.private_locator_schema_version
        logical_artifact_id = "runtime-evidence-$($evidence.RunId)"
        run_id = $evidence.RunId
        evidence_path = Join-Path $finalRoot 'runtime-evidence.v1.jsonl'
        build_manifest_path = Join-Path $finalRoot 'build-manifest.v1.json'
        producer_attestation_path = Join-Path $finalRoot 'producer-attestation.v1.json'
        evidence_digest = $evidenceDigest
        build_manifest_digest = $manifestDigest
        producer_attestation_digest = $producer.Digest
        sanitized = $sanitizedBase
    }
    $locator = New-SignedLocator $locatorPayload $locatorAuthority
    $locatorAuthority.Signer.Dispose()
    if ($null -ne $locatorAuthority.Key) { $locatorAuthority.Key.Dispose() }
    $locatorBytes = [Text.UTF8Encoding]::new($false).GetBytes(($locator | ConvertTo-Json -Depth 32))
    if ($locatorBytes.Length -gt $contracts.Sanitized.bounds.max_locator_bytes) { Fail 'BOUND_EXCEEDED' }
    Write-PrivateBytes (Join-Path $stagingRoot 'locator.v1.json') $locatorBytes
    $null = Resolve-SafeDestination $destination
    Move-Item -LiteralPath $stagingRoot -Destination $finalRoot
    $script:cleanupRoot = $finalRoot
    $script:diagnosticId = 'retained-reread'
    $sanitized = Read-VerifiedLocator (Join-Path $finalRoot 'locator.v1.json') $contracts
    $script:cleanupRoot = $null
    Write-Output ($sanitized | ConvertTo-Json -Compress -Depth 32)
    exit 0
}
catch {
    Remove-ExactCleanupTargets
    Write-Rejection
    exit 1
}
