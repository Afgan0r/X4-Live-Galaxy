Set-StrictMode -Version Latest
$boundedReaderPath = Join-Path $PSScriptRoot 'bounded-file.psm1'
Import-Module $boundedReaderPath -Force
$ErrorActionPreference = 'Stop'

$script:ProductionRootId = 'live-galaxy-owner-root-v1'
$script:ProductionRootSpkiSha256 = 'UNCONFIGURED'
$script:TestRootSpkiSha256 = '0db8b9c42e0b1a3504297060ed8ee79b690e640873a3a4745e49ed67698e12ea'
$script:Algorithm = 'ECDSA_P256_SHA256'
$script:OwnerOverridePurpose = 'owner-override'
$script:OwnerOverrideScope = 'known-failure:exact-finding'
$script:ProductionPolicyDigest = '6497cd18a4ee0286f0d566978c9225eaa168ba5d71f8fd7042a78507e1462854'
$script:CertificateFields = @(
    'schema_version', 'certificate_id', 'root_id', 'root_spki_sha256',
    'delegated_spki_sha256', 'windows_key_name', 'purpose', 'epoch',
    'scope', 'algorithm', 'not_before', 'not_after', 'policy_digest'
)
$script:OverridePayloadFields = @(
    'schema_version', 'override_id', 'authority_purpose',
    'delegation_certificate_id', 'dossier_id', 'dossier_digest',
    'finding_id', 'owner_decision_id', 'decision', 'rationale',
    'remaining_risk', 'issued_at', 'expires_at', 'nonce',
    'signature_algorithm'
)

Import-Module (Join-Path $PSScriptRoot 'producer-attestation.psm1') -Force

function New-AuthorityResult([string]$Status, [bool]$Ready = $false) {
    return [pscustomobject][ordered]@{
        schema_version = 'x4-owner-authority-result.v1'
        status = $Status
        authority_ready = $Ready
    }
}

function Get-Sha256Hex([byte[]]$Bytes) {
    return ([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes))).ToLowerInvariant()
}

function Test-CngUserPresencePolicy([Security.Cryptography.CngKey]$Key) {
    return Test-CngSigningKeyPolicy $Key
}

function Test-ProductionCngSigningKeyPolicy([Security.Cryptography.CngKey]$Key) {
    return Test-CngUserPresencePolicy $Key
}

function Get-CngUserPresenceStatus([Parameter(Mandatory)][string]$KeyName) {
    if (-not $IsWindows) { return New-AuthorityResult 'OWNER_AUTHORITY_PLATFORM_UNSUPPORTED' }
    try {
        $key = [Security.Cryptography.CngKey]::Open(
            $KeyName, [Security.Cryptography.CngProvider]::MicrosoftSoftwareKeyStorageProvider,
            [Security.Cryptography.CngKeyOpenOptions]::UserKey
        )
        try {
            if (-not (Test-CngUserPresencePolicy $key)) {
                return New-AuthorityResult 'OWNER_DELEGATED_KEY_USER_PRESENCE_REQUIRED'
            }
            return New-AuthorityResult 'OWNER_DELEGATED_KEY_USER_PRESENCE_VERIFIED' $true
        }
        finally { $key.Dispose() }
    }
    catch { return New-AuthorityResult 'OWNER_DELEGATED_KEY_UNCONFIGURED' }
}

function Assert-NoReparsePath([string]$Path) {
    $current = [IO.Path]::GetFullPath($Path)
    while (-not [string]::IsNullOrWhiteSpace($current)) {
        if (Test-Path -LiteralPath $current) {
            $segment = Get-Item -LiteralPath $current -Force
            if (($segment.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw [IO.IOException]::new('OWNER_ROOT_PATH_REJECTED')
            }
        }
        $parent = Split-Path -Parent $current
        if ([string]::IsNullOrWhiteSpace($parent) -or $parent -eq $current) { break }
        $current = $parent
    }
}

function Read-AuthorityJson([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw [IO.FileNotFoundException]::new('authority contract missing')
    }
    $read = Read-BoundedFile $Path 32768 'authority contract path rejected' `
        'authority contract path rejected' 'authority contract identity changed' `
        -ReparseCode 'OWNER_ROOT_PATH_REJECTED'
    return [Text.Encoding]::UTF8.GetString($read.Bytes) | ConvertFrom-Json
}

function Test-ExactAnchorPolicy($Anchor) {
    if ($Anchor.schema_version -ne 'x4-owner-root-anchor.v1' -or
        $Anchor.algorithm -ne $script:Algorithm -or
        $Anchor.policy_digest -ne $script:ProductionPolicyDigest) {
        return $false
    }
    $expectedEpochs = [ordered]@{
        'owner-override' = 1
        'candidate-producer' = 1
        'retention-locator' = 1
    }
    $expectedScopes = [ordered]@{
        'owner-override' = 'known-failure:exact-finding'
        'candidate-producer' = 'candidate-evidence:exact-build'
        'retention-locator' = 'retained-evidence:exact-run'
    }
    foreach ($purpose in $expectedEpochs.Keys) {
        if ($Anchor.accepted_epochs.PSObject.Properties.Name -notcontains $purpose -or
            [int]$Anchor.accepted_epochs.$purpose -ne $expectedEpochs[$purpose] -or
            $Anchor.scopes.PSObject.Properties.Name -notcontains $purpose -or
            [string]$Anchor.scopes.$purpose -ne $expectedScopes[$purpose]) {
            return $false
        }
    }
    return $true
}

function Get-OwnerAuthorityStatus {
    try {
        $anchorPath = Join-Path $PSScriptRoot 'contracts/owner-root-anchor.v1.json'
        $anchor = Read-AuthorityJson $anchorPath
        if ($anchor.root_id -ne $script:ProductionRootId -or -not (Test-ExactAnchorPolicy $anchor)) {
            return New-AuthorityResult 'OWNER_ROOT_PIN_MISMATCH'
        }
        if ($anchor.status -eq 'unconfigured') {
            if ($null -ne $anchor.root_spki_der_base64 -or $null -ne $anchor.root_spki_sha256) {
                return New-AuthorityResult 'OWNER_ROOT_PIN_MISMATCH'
            }
            return New-AuthorityResult 'OWNER_ROOT_UNCONFIGURED'
        }
        if ($anchor.status -ne 'configured' -or
            $script:ProductionRootSpkiSha256 -eq 'UNCONFIGURED' -or
            $anchor.root_spki_sha256 -ne $script:ProductionRootSpkiSha256) {
            return New-AuthorityResult 'OWNER_ROOT_PIN_MISMATCH'
        }
        [byte[]]$spki = [Convert]::FromBase64String([string]$anchor.root_spki_der_base64)
        if ((Get-Sha256Hex $spki) -ne $script:ProductionRootSpkiSha256) {
            return New-AuthorityResult 'OWNER_ROOT_PIN_MISMATCH'
        }
        return New-AuthorityResult 'OWNER_ROOT_VERIFIED' $true
    }
    catch {
        if ($_.Exception.Message -eq 'OWNER_ROOT_PATH_REJECTED') {
            return New-AuthorityResult 'OWNER_ROOT_PATH_REJECTED'
        }
        return New-AuthorityResult 'OWNER_ROOT_CONTRACT_INVALID'
    }
}

function ConvertTo-CertificateBytes($Certificate) {
    $builder = [Text.StringBuilder]::new()
    foreach ($field in $script:CertificateFields) {
        if ($Certificate.PSObject.Properties.Name -notcontains $field) {
            throw [IO.InvalidDataException]::new('delegated certificate field missing')
        }
        $rawValue = $Certificate.$field
        if ($rawValue -is [DateTime]) {
            $value = $rawValue.ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ', [Globalization.CultureInfo]::InvariantCulture)
        }
        elseif ($rawValue -is [DateTimeOffset]) {
            $value = $rawValue.ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ', [Globalization.CultureInfo]::InvariantCulture)
        }
        else {
            $value = [string]$rawValue
        }
        [void]$builder.Append($field).Append('=').Append($value.Length).Append(':').Append($value).Append("`n")
    }
    return [Text.Encoding]::UTF8.GetBytes($builder.ToString())
}

function ConvertTo-CanonicalFieldBytes($Value, [string[]]$Fields) {
    $builder = [Text.StringBuilder]::new()
    foreach ($field in $Fields) {
        if ($Value.PSObject.Properties.Name -notcontains $field) {
            throw [IO.InvalidDataException]::new('canonical field missing')
        }
        $rawValue = $Value.$field
        if ($rawValue -is [DateTime]) {
            $textValue = $rawValue.ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ', [Globalization.CultureInfo]::InvariantCulture)
        }
        elseif ($rawValue -is [DateTimeOffset]) {
            $textValue = $rawValue.ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ', [Globalization.CultureInfo]::InvariantCulture)
        }
        else { $textValue = [string]$rawValue }
        [void]$builder.Append($field).Append('=').Append($textValue.Length).Append(':').Append($textValue).Append("`n")
    }
    return [Text.Encoding]::UTF8.GetBytes($builder.ToString())
}

function ConvertTo-OwnerOverridePayloadBytes($Override) {
    return ConvertTo-CanonicalFieldBytes $Override $script:OverridePayloadFields
}

function Test-ExactOwnerOverrideCore($Override, $Certificate, [string]$ExpectedDossierId, [string]$ExpectedDossierDigest, [string[]]$KnownFindingIds) {
    try {
        $expectedFields = @($script:OverridePayloadFields) + @('payload_digest', 'signature_base64')
        $actualFields = @($Override.PSObject.Properties.Name | Sort-Object)
        if (($actualFields -join '|') -ne (@($expectedFields | Sort-Object) -join '|')) {
            return New-AuthorityResult 'MISSING_REQUIRED_FIELD'
        }
        if ($Override.schema_version -ne 'x4-owner-override.v1' -or
            $Override.authority_purpose -ne $script:OwnerOverridePurpose -or
            $Override.delegation_certificate_id -ne $Certificate.certificate_id -or
            $Override.signature_algorithm -ne $script:Algorithm) {
            return New-AuthorityResult 'OWNER_OVERRIDE_AUTHORITY_MISMATCH'
        }
        [byte[]]$payloadBytes = ConvertTo-OwnerOverridePayloadBytes $Override
        $payloadDigest = Get-Sha256Hex $payloadBytes
        if ($Override.payload_digest -ne $payloadDigest) {
            return New-AuthorityResult 'OWNER_OVERRIDE_PAYLOAD_DIGEST_MISMATCH'
        }
        [byte[]]$delegatedSpki = [Convert]::FromBase64String([string]$Certificate.delegated_spki_der_base64)
        [byte[]]$signature = [Convert]::FromBase64String([string]$Override.signature_base64)
        $verifier = [Security.Cryptography.ECDsa]::Create()
        try {
            $read = 0
            $verifier.ImportSubjectPublicKeyInfo($delegatedSpki, [ref]$read)
            $signatureValid = $verifier.VerifyData(
                $payloadBytes,
                $signature,
                [Security.Cryptography.HashAlgorithmName]::SHA256,
                [Security.Cryptography.DSASignatureFormat]::IeeeP1363FixedFieldConcatenation
            )
        }
        finally { $verifier.Dispose() }
        if (-not $signatureValid) {
            return New-AuthorityResult 'OWNER_OVERRIDE_SIGNATURE_INVALID'
        }
        foreach ($textField in @('override_id', 'dossier_id', 'finding_id', 'owner_decision_id', 'rationale', 'remaining_risk', 'nonce')) {
            $text = [string]$Override.$textField
            if ([string]::IsNullOrWhiteSpace($text) -or $text.Length -gt 256) {
                return New-AuthorityResult 'INVALID_FIELD_VALUE'
            }
        }
        foreach ($idField in @('override_id', 'dossier_id', 'finding_id', 'owner_decision_id')) {
            if ([string]$Override.$idField -notmatch '^[a-z0-9][a-z0-9._-]{0,127}$') {
                return New-AuthorityResult 'INVALID_FIELD_VALUE'
            }
        }
        if ([string]$Override.nonce -notmatch '^[a-z0-9][a-z0-9._-]{0,127}$') {
            return New-AuthorityResult 'INVALID_FIELD_VALUE'
        }
        if ($Override.dossier_id -ne $ExpectedDossierId -or $KnownFindingIds -notcontains $Override.finding_id) {
            return New-AuthorityResult 'OVERRIDE_SCOPE_MISMATCH'
        }
        if ($Override.dossier_digest -ne $ExpectedDossierDigest) {
            return New-AuthorityResult 'OVERRIDE_DIGEST_MISMATCH'
        }
        if ($Override.decision -ne 'accept-risk') {
            return New-AuthorityResult 'INVALID_OWNER_DECISION'
        }
        $issued = [DateTimeOffset]::Parse([string]$Override.issued_at, [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::AssumeUniversal)
        $expiry = [DateTimeOffset]::Parse([string]$Override.expires_at, [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::AssumeUniversal)
        $now = [DateTimeOffset]::UtcNow
        if ($issued -gt $now.AddMinutes(5)) { return New-AuthorityResult 'OVERRIDE_ISSUED_IN_FUTURE' }
        if ($expiry -le $now) { return New-AuthorityResult 'OVERRIDE_EXPIRED' }
        if ($expiry -gt $issued.AddDays(90)) { return New-AuthorityResult 'OVERRIDE_EXPIRY_OUT_OF_RANGE' }
        $result = New-AuthorityResult 'OWNER_OVERRIDE_VERIFIED' $true
        $result | Add-Member -NotePropertyName overridden_finding_ids -NotePropertyValue @([string]$Override.finding_id)
        return $result
    }
    catch {
        return New-AuthorityResult 'OWNER_OVERRIDE_INVALID'
    }
}

function New-SignedOwnerOverrideEnvelope($Payload, [Security.Cryptography.ECDsa]$Signer) {
    $actual = @($Payload.PSObject.Properties.Name | Sort-Object)
    if (($actual -join '|') -ne (@($script:OverridePayloadFields | Sort-Object) -join '|')) {
        throw [IO.InvalidDataException]::new('owner override payload fields invalid')
    }
    [byte[]]$bytes = ConvertTo-OwnerOverridePayloadBytes $Payload
    [byte[]]$signature = $Signer.SignData(
        $bytes,
        [Security.Cryptography.HashAlgorithmName]::SHA256,
        [Security.Cryptography.DSASignatureFormat]::IeeeP1363FixedFieldConcatenation
    )
    $envelope = [ordered]@{}
    foreach ($field in $script:OverridePayloadFields) { $envelope[$field] = $Payload.$field }
    $envelope.payload_digest = Get-Sha256Hex $bytes
    $envelope.signature_base64 = [Convert]::ToBase64String($signature)
    return [pscustomobject]$envelope
}

function Test-DelegatedCertificateCore($Anchor, $Certificate, [string]$ExpectedRootFingerprint) {
    try {
        if ($Certificate.purpose -ne $script:OwnerOverridePurpose) {
            return New-AuthorityResult 'OWNER_DELEGATION_PURPOSE_MISMATCH'
        }
        if ($Certificate.scope -ne $script:OwnerOverrideScope) {
            return New-AuthorityResult 'OWNER_DELEGATION_SCOPE_MISMATCH'
        }
        $acceptedEpoch = [int]$Anchor.accepted_epochs.'owner-override'
        if ([int]$Certificate.epoch -lt $acceptedEpoch) {
            return New-AuthorityResult 'OWNER_DELEGATION_EPOCH_ROLLBACK'
        }
        if ([int]$Certificate.epoch -ne $acceptedEpoch) {
            return New-AuthorityResult 'OWNER_DELEGATION_EPOCH_UNACCEPTED'
        }
        if ($Certificate.schema_version -ne 'x4-delegated-purpose-certificate.v1' -or
            $Certificate.root_id -ne $Anchor.root_id -or
            $Certificate.root_spki_sha256 -ne $ExpectedRootFingerprint -or
            $Certificate.algorithm -ne $script:Algorithm) {
            return New-AuthorityResult 'OWNER_DELEGATION_INVALID'
        }
        $policyText = "$($Certificate.purpose)|$($Certificate.epoch)|$($Certificate.scope)"
        if ($Certificate.policy_digest -ne (Get-Sha256Hex ([Text.Encoding]::UTF8.GetBytes($policyText)))) {
            return New-AuthorityResult 'OWNER_DELEGATION_POLICY_MISMATCH'
        }
        $now = [DateTimeOffset]::UtcNow
        $notBefore = if ($Certificate.not_before -is [DateTime]) {
            [DateTimeOffset]::new($Certificate.not_before.ToUniversalTime())
        }
        else {
            [DateTimeOffset]::ParseExact($Certificate.not_before, 'yyyy-MM-ddTHH:mm:ssZ', [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::AssumeUniversal)
        }
        $notAfter = if ($Certificate.not_after -is [DateTime]) {
            [DateTimeOffset]::new($Certificate.not_after.ToUniversalTime())
        }
        else {
            [DateTimeOffset]::ParseExact($Certificate.not_after, 'yyyy-MM-ddTHH:mm:ssZ', [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::AssumeUniversal)
        }
        if ($now -lt $notBefore -or $now -ge $notAfter) {
            return New-AuthorityResult 'OWNER_DELEGATION_EXPIRED'
        }
        [byte[]]$delegatedSpki = [Convert]::FromBase64String([string]$Certificate.delegated_spki_der_base64)
        if ((Get-Sha256Hex $delegatedSpki) -ne $Certificate.delegated_spki_sha256) {
            return New-AuthorityResult 'OWNER_DELEGATED_KEY_MISMATCH'
        }
        [byte[]]$rootSpki = [Convert]::FromBase64String([string]$Anchor.root_spki_der_base64)
        if ((Get-Sha256Hex $rootSpki) -ne $ExpectedRootFingerprint) {
            return New-AuthorityResult 'OWNER_ROOT_PIN_MISMATCH'
        }
        [byte[]]$signature = [Convert]::FromBase64String([string]$Certificate.root_signature_base64)
        $root = [Security.Cryptography.ECDsa]::Create()
        try {
            $read = 0
            $root.ImportSubjectPublicKeyInfo($rootSpki, [ref]$read)
            $verified = $root.VerifyData(
                (ConvertTo-CertificateBytes $Certificate),
                $signature,
                [Security.Cryptography.HashAlgorithmName]::SHA256,
                [Security.Cryptography.DSASignatureFormat]::IeeeP1363FixedFieldConcatenation
            )
        }
        finally { $root.Dispose() }
        if (-not $verified) {
            return New-AuthorityResult 'OWNER_DELEGATION_ROOT_SIGNATURE_INVALID'
        }
        return New-AuthorityResult 'OWNER_DELEGATION_VERIFIED' $true
    }
    catch {
        return New-AuthorityResult 'OWNER_DELEGATION_INVALID'
    }
}

function Get-OwnerOverrideDelegationStatus {
    $rootStatus = Get-OwnerAuthorityStatus
    if (-not $rootStatus.authority_ready) { return $rootStatus }
    try {
        $anchor = Read-AuthorityJson (Join-Path $PSScriptRoot 'contracts/owner-root-anchor.v1.json')
        $certificate = Read-AuthorityJson (Join-Path $PSScriptRoot 'contracts/delegated-purpose-certificate.v1.json')
        if ($certificate.status -eq 'unconfigured') {
            return New-AuthorityResult 'OWNER_DELEGATED_KEY_UNCONFIGURED'
        }
        return Test-DelegatedCertificateCore $anchor $certificate $script:ProductionRootSpkiSha256
    }
    catch {
        return New-AuthorityResult 'OWNER_DELEGATION_INVALID'
    }
}

function Get-OwnerOverrideSigningStatus {
    if (-not $IsWindows) {
        return New-AuthorityResult 'OWNER_AUTHORITY_PLATFORM_UNSUPPORTED'
    }
    $delegation = Get-OwnerOverrideDelegationStatus
    if (-not $delegation.authority_ready) { return $delegation }
    try {
        $certificate = Read-AuthorityJson (Join-Path $PSScriptRoot 'contracts/delegated-purpose-certificate.v1.json')
        $key = [Security.Cryptography.CngKey]::Open(
            [string]$certificate.windows_key_name,
            [Security.Cryptography.CngProvider]::MicrosoftSoftwareKeyStorageProvider,
            [Security.Cryptography.CngKeyOpenOptions]::UserKey
        )
        try {
            $disallowed = [Security.Cryptography.CngExportPolicies]::AllowExport -bor
                [Security.Cryptography.CngExportPolicies]::AllowPlaintextExport
            if (($key.ExportPolicy -band $disallowed) -ne 0) {
                return New-AuthorityResult 'OWNER_DELEGATED_KEY_EXPORTABLE'
            }
            if (-not (Test-CngUserPresencePolicy $key)) {
                return New-AuthorityResult 'OWNER_DELEGATED_KEY_USER_PRESENCE_REQUIRED'
            }
            $signer = [Security.Cryptography.ECDsaCng]::new($key)
            try { [byte[]]$spki = $signer.ExportSubjectPublicKeyInfo() }
            finally { $signer.Dispose() }
            if ((Get-Sha256Hex $spki) -ne $certificate.delegated_spki_sha256) {
                return New-AuthorityResult 'OWNER_DELEGATED_KEY_MISMATCH'
            }
        }
        finally { $key.Dispose() }
        return New-AuthorityResult 'OWNER_DELEGATED_KEY_VERIFIED' $true
    }
    catch [Security.Cryptography.CryptographicException] {
        return New-AuthorityResult 'OWNER_DELEGATED_KEY_UNCONFIGURED'
    }
    catch {
        return New-AuthorityResult 'OWNER_DELEGATED_KEY_ACCESS_FAILED'
    }
}

function Test-TestOnlyFixturePath([string]$FixturePath) {
    $fixtureRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot 'tests/fixtures'))
    $resolved = [IO.Path]::GetFullPath($FixturePath)
    return $resolved.StartsWith($fixtureRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase) -and
        [IO.Path]::GetFileName($resolved) -eq 'test-owner-root-fixture.v1.json'
}

function Test-TestOnlyDelegatedCertificate {
    param([Parameter(Mandatory = $true)][string]$FixturePath)
    if (-not (Test-TestOnlyFixturePath $FixturePath)) {
        return New-AuthorityResult 'TEST_AUTHORITY_PATH_REJECTED'
    }
    try {
        $fixture = Read-AuthorityJson $FixturePath
        return Test-TestOnlyDelegatedCertificateObject -Fixture $fixture
    }
    catch {
        return New-AuthorityResult 'TEST_AUTHORITY_INVALID'
    }
}

function Test-TestOnlyDelegatedCertificateObject {
    param([Parameter(Mandatory = $true)]$Fixture)
    if ($Fixture.schema_version -ne 'x4-test-owner-root-fixture.v1' -or
        $Fixture.marker -ne 'TEST-ONLY-NEVER-PRODUCTION' -or
        $Fixture.root_anchor.root_id -ne 'TEST-ONLY-owner-root' -or
        $Fixture.root_anchor.root_spki_sha256 -ne $script:TestRootSpkiSha256) {
        return New-AuthorityResult 'TEST_AUTHORITY_INVALID'
    }
    return Test-DelegatedCertificateCore $Fixture.root_anchor $Fixture.delegated_certificate $script:TestRootSpkiSha256
}

function New-TestOnlyOwnerOverrideEnvelope {
    param(
        [Parameter(Mandatory = $true)]$Payload,
        [Parameter(Mandatory = $true)][string]$FixturePath
    )
    $delegation = Test-TestOnlyDelegatedCertificate -FixturePath $FixturePath
    if (-not $delegation.authority_ready) { throw $delegation.status }
    $fixture = Read-AuthorityJson $FixturePath
    [byte[]]$privateKey = [Convert]::FromBase64String([string]$fixture.delegated_private_pkcs8_base64)
    $signer = [Security.Cryptography.ECDsa]::Create()
    try {
        $read = 0
        $signer.ImportPkcs8PrivateKey($privateKey, [ref]$read)
        return New-SignedOwnerOverrideEnvelope $Payload $signer
    }
    finally { $signer.Dispose() }
}

function Test-TestOnlyExactOwnerOverride {
    param(
        [Parameter(Mandatory = $true)]$Override,
        [Parameter(Mandatory = $true)][string]$FixturePath,
        [Parameter(Mandatory = $true)][string]$ExpectedDossierId,
        [Parameter(Mandatory = $true)][string]$ExpectedDossierDigest,
        [Parameter(Mandatory = $true)][string[]]$KnownFindingIds
    )
    $delegation = Test-TestOnlyDelegatedCertificate -FixturePath $FixturePath
    if (-not $delegation.authority_ready) { return $delegation }
    $fixture = Read-AuthorityJson $FixturePath
    return Test-ExactOwnerOverrideCore $Override $fixture.delegated_certificate $ExpectedDossierId $ExpectedDossierDigest $KnownFindingIds
}

function Test-ProductionExactOwnerOverride {
    param(
        [Parameter(Mandatory = $true)]$Override,
        [Parameter(Mandatory = $true)][string]$ExpectedDossierId,
        [Parameter(Mandatory = $true)][string]$ExpectedDossierDigest,
        [Parameter(Mandatory = $true)][string[]]$KnownFindingIds
    )
    $delegation = Get-OwnerOverrideDelegationStatus
    if (-not $delegation.authority_ready) { return $delegation }
    $certificate = Read-AuthorityJson (Join-Path $PSScriptRoot 'contracts/delegated-purpose-certificate.v1.json')
    return Test-ExactOwnerOverrideCore $Override $certificate $ExpectedDossierId $ExpectedDossierDigest $KnownFindingIds
}

function New-ProductionOwnerOverrideEnvelope {
    param([Parameter(Mandatory = $true)]$Payload)
    $status = Get-OwnerOverrideSigningStatus
    if (-not $status.authority_ready) { return $status }
    $certificate = Read-AuthorityJson (Join-Path $PSScriptRoot 'contracts/delegated-purpose-certificate.v1.json')
    try {
        $key = [Security.Cryptography.CngKey]::Open(
            [string]$certificate.windows_key_name,
            [Security.Cryptography.CngProvider]::MicrosoftSoftwareKeyStorageProvider,
            [Security.Cryptography.CngKeyOpenOptions]::UserKey
        )
        try {
            if (-not (Test-CngUserPresencePolicy $key)) {
                return New-AuthorityResult 'OWNER_DELEGATED_KEY_USER_PRESENCE_REQUIRED'
            }
            $signer = [Security.Cryptography.ECDsaCng]::new($key)
            try { return New-SignedOwnerOverrideEnvelope $Payload $signer }
            finally { $signer.Dispose() }
        }
        finally { $key.Dispose() }
    }
    catch [Security.Cryptography.CryptographicException] {
        return New-AuthorityResult 'OWNER_DELEGATED_KEY_ACCESS_FAILED'
    }
}

Export-ModuleMember -Function @(
    'Get-OwnerAuthorityStatus',
    'Get-OwnerOverrideDelegationStatus',
    'Get-OwnerOverrideSigningStatus',
    'Get-CngUserPresenceStatus',
    'Test-ProductionCngSigningKeyPolicy',
    'Test-TestOnlyDelegatedCertificate',
    'Test-TestOnlyDelegatedCertificateObject',
    'New-TestOnlyOwnerOverrideEnvelope',
    'Test-TestOnlyExactOwnerOverride',
    'Test-ProductionExactOwnerOverride',
    'New-ProductionOwnerOverrideEnvelope'
)
