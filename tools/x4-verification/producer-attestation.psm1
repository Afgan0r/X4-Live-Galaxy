Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:PayloadFields = @(
    'schema_version', 'authority_purpose', 'delegation_certificate_id',
    'protocol_version', 'purpose', 'epoch', 'scope', 'dispatcher_digest',
    'adapter_digest', 'attestation_module_digest', 'worker_digest', 'launcher_digest',
    'worker_protocol_digest', 'runtime_evidence_schema_digest', 'build_id',
    'package_conformance_digest', 'matrix_digest', 'run_id', 'candidate_ids',
    'evidence_digest', 'started_at', 'completed_at', 'classification', 'nonce',
    'expires_at', 'signature_algorithm'
)
$script:EnvelopeFields = @(
    'schema_version', 'certificate', 'payload', 'payload_digest',
    'signature_base64'
)
$script:DigestFields = @(
    'dispatcher_digest', 'adapter_digest', 'attestation_module_digest', 'worker_digest', 'launcher_digest',
    'worker_protocol_digest', 'runtime_evidence_schema_digest',
    'package_conformance_digest', 'matrix_digest', 'evidence_digest'
)

function Get-Sha256Hex([byte[]]$Bytes) {
    return [Convert]::ToHexString(
        [Security.Cryptography.SHA256]::HashData($Bytes)
    ).ToLowerInvariant()
}

function Test-CngSigningKeyPolicy([Security.Cryptography.CngKey]$Key) {
    $disallowed = [Security.Cryptography.CngExportPolicies]::AllowExport -bor
        [Security.Cryptography.CngExportPolicies]::AllowPlaintextExport
    $required = [Security.Cryptography.CngUIProtectionLevels]::ForceHighProtection
    return $Key.Provider.Provider -ceq [Security.Cryptography.CngProvider]::MicrosoftSoftwareKeyStorageProvider.Provider -and
        ($Key.ExportPolicy -band $disallowed) -eq 0 -and
        ($Key.UIPolicy.ProtectionLevel -band $required) -eq $required
}

function ConvertTo-CanonicalValue($Value) {
    if ($null -eq $Value -or $Value -is [string] -or $Value -is [bool] -or
        $Value -is [byte] -or $Value -is [int16] -or $Value -is [int32] -or
        $Value -is [int64] -or $Value -is [decimal] -or $Value -is [double]) {
        return $Value
    }
    if ($Value -is [Collections.IDictionary]) {
        $result = [ordered]@{}
        foreach ($key in @($Value.Keys | Sort-Object)) {
            $result[$key] = ConvertTo-CanonicalValue $Value[$key]
        }
        return $result
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
    return ConvertTo-CanonicalValue $Value | ConvertTo-Json -Compress -Depth 32
}

function Assert-ExactFields($Value, [string[]]$Expected, [string]$Code) {
    $actual = if ($Value -is [Collections.IDictionary]) {
        @($Value.Keys | Sort-Object)
    }
    else { @($Value.PSObject.Properties.Name | Sort-Object) }
    if (($actual -join '|') -ne (@($Expected | Sort-Object) -join '|')) {
        throw [IO.InvalidDataException]::new($Code)
    }
}

function Test-CandidateProducerPayload {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Payload,
        [Parameter(Mandatory)][string]$CertificateId,
        [Parameter(Mandatory)][int]$Epoch,
        [Parameter(Mandatory)][string]$Scope,
        [Parameter(Mandatory)][DateTimeOffset]$Now
    )

    Assert-ExactFields $Payload $script:PayloadFields 'PRODUCER_PAYLOAD_FIELDS_INVALID'
    if ($Payload.schema_version -ne 'candidate-producer-envelope.v1' -or
        $Payload.authority_purpose -ne 'candidate-producer' -or
        $Payload.purpose -ne 'candidate-producer' -or
        $Payload.protocol_version -ne 'candidate-worker.v1' -or
        $Payload.delegation_certificate_id -cne $CertificateId -or
        [int]$Payload.epoch -ne $Epoch -or $Payload.scope -cne $Scope -or
        $Payload.classification -ne 'authenticated-local-contract' -or
        $Payload.signature_algorithm -ne 'ECDSA_P256_SHA256') {
        throw [IO.InvalidDataException]::new('PRODUCER_PAYLOAD_POLICY_INVALID')
    }
    foreach ($name in $script:DigestFields) {
        if ([string]$Payload.$name -notmatch '^[a-f0-9]{64}$') {
            throw [IO.InvalidDataException]::new('PRODUCER_PAYLOAD_DIGEST_INVALID')
        }
    }
    if ([string]$Payload.nonce -notmatch '^[a-f0-9]{32}$') {
        throw [IO.InvalidDataException]::new('PRODUCER_NONCE_INVALID')
    }
    $candidateIds = @($Payload.candidate_ids)
    if ($candidateIds.Count -lt 1 -or $candidateIds.Count -gt 7 -or
        ($candidateIds -join '|') -ne (@($candidateIds | Sort-Object -Unique) -join '|')) {
        throw [IO.InvalidDataException]::new('PRODUCER_CANDIDATES_INVALID')
    }
    try {
        $started = [DateTimeOffset]::ParseExact(
            [string]$Payload.started_at, 'O', [Globalization.CultureInfo]::InvariantCulture
        )
        $completed = [DateTimeOffset]::ParseExact(
            [string]$Payload.completed_at, 'O', [Globalization.CultureInfo]::InvariantCulture
        )
        $expires = [DateTimeOffset]::ParseExact(
            [string]$Payload.expires_at, 'O', [Globalization.CultureInfo]::InvariantCulture
        )
    }
    catch { throw [IO.InvalidDataException]::new('PRODUCER_TIME_INVALID') }
    if ($Now -ge $expires) {
        throw [IO.InvalidDataException]::new('PRODUCER_ATTESTATION_EXPIRED')
    }
    $verificationCeiling = $Now.AddMinutes(5)
    if ($completed -lt $started -or $completed -gt $verificationCeiling -or
        $expires -le $verificationCeiling -or
        ($expires - $started) -gt [TimeSpan]::FromHours(24) -or
        $started -gt $verificationCeiling) {
        throw [IO.InvalidDataException]::new('PRODUCER_CHRONOLOGY_INVALID')
    }
    return $true
}

function New-CandidateProducerAttestation {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Payload,
        [Parameter(Mandatory)]$Certificate,
        [Parameter(Mandatory)][Security.Cryptography.ECDsa]$Signer
    )
    $null = Test-CandidateProducerPayload -Payload $Payload `
        -CertificateId ([string]$Certificate.certificate_id) `
        -Epoch ([int]$Certificate.epoch) -Scope ([string]$Certificate.scope) `
        -Now ([DateTimeOffset]::UtcNow)
    [byte[]]$payloadBytes = [Text.UTF8Encoding]::new($false).GetBytes(
        (ConvertTo-CanonicalJson $Payload)
    )
    [byte[]]$signature = $Signer.SignData(
        $payloadBytes, [Security.Cryptography.HashAlgorithmName]::SHA256,
        [Security.Cryptography.DSASignatureFormat]::IeeeP1363FixedFieldConcatenation
    )
    return [ordered]@{
        schema_version = 'candidate-producer-attestation.v1'
        certificate = $Certificate
        payload = $Payload
        payload_digest = Get-Sha256Hex $payloadBytes
        signature_base64 = [Convert]::ToBase64String($signature)
    }
}

function Test-CandidateProducerEnvelopeFields($Envelope) {
    Assert-ExactFields $Envelope $script:EnvelopeFields 'PRODUCER_ENVELOPE_FIELDS_INVALID'
    if ($Envelope.schema_version -ne 'candidate-producer-attestation.v1') {
        throw [IO.InvalidDataException]::new('PRODUCER_ENVELOPE_SCHEMA_INVALID')
    }
    return $true
}

Export-ModuleMember -Function @(
    'ConvertTo-CanonicalJson',
    'New-CandidateProducerAttestation',
    'Test-CandidateProducerEnvelopeFields',
    'Test-CandidateProducerPayload',
    'Test-CngSigningKeyPolicy'
)
