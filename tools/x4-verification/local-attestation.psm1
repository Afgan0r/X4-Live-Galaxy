Set-StrictMode -Version Latest
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

function Read-AuthorityJson([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw [IO.FileNotFoundException]::new('authority contract missing')
    }
    $item = Get-Item -LiteralPath $Path -Force
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or $item.Length -gt 32768) {
        throw [IO.InvalidDataException]::new('authority contract path rejected')
    }
    return Get-Content -LiteralPath $Path -Raw -Encoding utf8 | ConvertFrom-Json
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

Export-ModuleMember -Function @(
    'Get-OwnerAuthorityStatus',
    'Get-OwnerOverrideDelegationStatus',
    'Get-OwnerOverrideSigningStatus',
    'Test-TestOnlyDelegatedCertificate',
    'Test-TestOnlyDelegatedCertificateObject'
)
