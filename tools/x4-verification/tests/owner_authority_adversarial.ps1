[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$toolRoot = Split-Path -Parent $PSScriptRoot
$modulePath = Join-Path $toolRoot 'local-attestation.psm1'
$admissionPath = Join-Path $toolRoot 'x4-admission.ps1'
$signerPath = Join-Path $toolRoot 'new-owner-override.ps1'
$fixturePath = Join-Path $PSScriptRoot 'fixtures/test-owner-root-fixture.v1.json'
$certificatePath = Join-Path $toolRoot 'contracts/delegated-purpose-certificate.v1.json'
$producerModulePath = Join-Path $toolRoot 'producer-attestation.psm1'

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Assert-Code($Result, [string]$Code, [string]$Label) {
    Assert-True ($Result.status -eq $Code) "$Label returned $($Result.status), expected $Code."
}

function ConvertTo-CanonicalBytes($Value) {
    $fields = @('schema_version', 'override_id', 'authority_purpose', 'delegation_certificate_id', 'dossier_id', 'dossier_digest', 'finding_id', 'owner_decision_id', 'decision', 'rationale', 'remaining_risk', 'issued_at', 'expires_at', 'nonce', 'signature_algorithm')
    $builder = [Text.StringBuilder]::new()
    foreach ($field in $fields) {
        $raw = $Value.$field
        if ($raw -is [DateTime]) { $text = $raw.ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ') }
        else { $text = [string]$raw }
        [void]$builder.Append($field).Append('=').Append($text.Length).Append(':').Append($text).Append("`n")
    }
    return [Text.Encoding]::UTF8.GetBytes($builder.ToString())
}

foreach ($path in @($modulePath, $admissionPath, $signerPath, $fixturePath, $certificatePath)) {
    Assert-True (Test-Path -LiteralPath $path -PathType Leaf) 'Held-out authority artifact is missing.'
}
Import-Module $modulePath -Force
$fixture = Get-Content -LiteralPath $fixturePath -Raw -Encoding utf8 | ConvertFrom-Json

$payload = [pscustomobject][ordered]@{
    schema_version = 'x4-owner-override.v1'
    override_id = 'owner-override-held-out'
    authority_purpose = 'owner-override'
    delegation_certificate_id = 'test-only-owner-override-delegation'
    dossier_id = 'held-out-dossier'
    dossier_digest = 'a' * 64
    finding_id = 'held-out-finding-one'
    owner_decision_id = 'held-out-decision'
    decision = 'accept-risk'
    rationale = 'TEST-ONLY held-out exact acceptance.'
    remaining_risk = 'TEST-ONLY known failure remains exact and bounded.'
    issued_at = [DateTimeOffset]::UtcNow.AddMinutes(-1).ToString('yyyy-MM-ddTHH:mm:ssZ')
    expires_at = [DateTimeOffset]::UtcNow.AddDays(1).ToString('yyyy-MM-ddTHH:mm:ssZ')
    nonce = 'held-out-nonce-one'
    signature_algorithm = 'ECDSA_P256_SHA256'
}
$signed = New-TestOnlyOwnerOverrideEnvelope -Payload $payload -FixturePath $fixturePath

$sameShape = $signed | ConvertTo-Json -Depth 12 | ConvertFrom-Json
$sameShape.rationale = 'TEST-ONLY same length altered text!'
$sameShape.payload_digest = ([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData((ConvertTo-CanonicalBytes $sameShape)))).ToLowerInvariant()
$sameShapeResult = Test-TestOnlyExactOwnerOverride -Override $sameShape -FixturePath $fixturePath -ExpectedDossierId 'held-out-dossier' -ExpectedDossierDigest ('a' * 64) -KnownFindingIds @('held-out-finding-one')
Assert-Code $sameShapeResult 'OWNER_OVERRIDE_SIGNATURE_INVALID' 'same-shape digest recomputation'

$secondFinding = $signed | ConvertTo-Json -Depth 12 | ConvertFrom-Json
$secondFinding.finding_id = 'held-out-finding-two'
$secondFinding.payload_digest = ([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData((ConvertTo-CanonicalBytes $secondFinding)))).ToLowerInvariant()
$secondResult = Test-TestOnlyExactOwnerOverride -Override $secondFinding -FixturePath $fixturePath -ExpectedDossierId 'held-out-dossier' -ExpectedDossierDigest ('a' * 64) -KnownFindingIds @('held-out-finding-one', 'held-out-finding-two')
Assert-Code $secondResult 'OWNER_OVERRIDE_SIGNATURE_INVALID' 'signature transplant to second finding'

$purpose = $fixture | ConvertTo-Json -Depth 16 | ConvertFrom-Json
$purpose.delegated_certificate.purpose = 'retention-locator'
Assert-Code (Test-TestOnlyDelegatedCertificateObject -Fixture $purpose) 'OWNER_DELEGATION_PURPOSE_MISMATCH' 'delegation purpose confusion'
$epoch = $fixture | ConvertTo-Json -Depth 16 | ConvertFrom-Json
$epoch.delegated_certificate.epoch = 0
Assert-Code (Test-TestOnlyDelegatedCertificateObject -Fixture $epoch) 'OWNER_DELEGATION_EPOCH_ROLLBACK' 'delegation epoch rollback'
$stalePolicy = $fixture | ConvertTo-Json -Depth 16 | ConvertFrom-Json
$stalePolicy.delegated_certificate.policy_digest = '0' * 64
Assert-Code (Test-TestOnlyDelegatedCertificateObject -Fixture $stalePolicy) 'OWNER_DELEGATION_POLICY_MISMATCH' 'stale delegation policy'
$substituted = $fixture | ConvertTo-Json -Depth 16 | ConvertFrom-Json
$substituted.delegated_certificate.delegated_spki_sha256 = '0' * 64
Assert-Code (Test-TestOnlyDelegatedCertificateObject -Fixture $substituted) 'OWNER_DELEGATED_KEY_MISMATCH' 'delegated key substitution'

foreach ($productionCommand in @((Get-Command $admissionPath), (Get-Command $signerPath))) {
    foreach ($forbidden in @('AnchorPath', 'RootPath', 'CertificatePath', 'PublicKeyPath', 'KeyName', 'TestRootPath', 'TestMode')) {
        Assert-True ($productionCommand.Parameters.Keys -notcontains $forbidden) "Production CLI exposes forbidden $forbidden input."
    }
}

function Invoke-CopiedAnchorProbe($Anchor) {
    $copyRoot = Join-Path ([IO.Path]::GetTempPath()) ("live-galaxy-anchor-probe-{0}" -f [guid]::NewGuid().ToString('N'))
    [void](New-Item -ItemType Directory -Path (Join-Path $copyRoot 'contracts') -Force)
    try {
        Copy-Item -LiteralPath $modulePath -Destination (Join-Path $copyRoot 'local-attestation.psm1')
        Copy-Item -LiteralPath $producerModulePath -Destination (Join-Path $copyRoot 'producer-attestation.psm1')
        $Anchor | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $copyRoot 'contracts/owner-root-anchor.v1.json') -Encoding utf8NoBOM
        Copy-Item -LiteralPath $certificatePath -Destination (Join-Path $copyRoot 'contracts/delegated-purpose-certificate.v1.json')
        $probe = "Import-Module '$($copyRoot.Replace("'", "''"))\local-attestation.psm1' -Force; Get-OwnerAuthorityStatus | ConvertTo-Json -Compress"
        $output = & pwsh -NoProfile -Command $probe
        Assert-True ($LASTEXITCODE -eq 0) 'Copied-anchor probe did not complete.'
        return $output | ConvertFrom-Json
    }
    finally { Remove-Item -LiteralPath $copyRoot -Recurse -Force }
}

$testRootSwap = $fixture.root_anchor | ConvertTo-Json -Depth 8 | ConvertFrom-Json
$testRootSwap.root_id = 'live-galaxy-owner-root-v1'
$testRootSwap | Add-Member -NotePropertyName policy_digest -NotePropertyValue '6497cd18a4ee0286f0d566978c9225eaa168ba5d71f8fd7042a78507e1462854'
Assert-Code (Invoke-CopiedAnchorProbe $testRootSwap) 'OWNER_ROOT_PIN_MISMATCH' 'TEST-ONLY repository root swap'

$freshKey = [Security.Cryptography.ECDsa]::Create()
try {
    [byte[]]$freshSpki = $freshKey.ExportSubjectPublicKeyInfo()
    $freshRoot = $testRootSwap | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $freshRoot.root_spki_der_base64 = [Convert]::ToBase64String($freshSpki)
    $freshRoot.root_spki_sha256 = ([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($freshSpki))).ToLowerInvariant()
    Assert-Code (Invoke-CopiedAnchorProbe $freshRoot) 'OWNER_ROOT_PIN_MISMATCH' 'fresh self-created production root'
}
finally { $freshKey.Dispose() }

$junctionRoot = Join-Path ([IO.Path]::GetTempPath()) ("live-galaxy-authority-junction-{0}" -f [guid]::NewGuid().ToString('N'))
$targetRoot = Join-Path $junctionRoot 'swapped'
$probeRoot = Join-Path $junctionRoot 'probe'
[void](New-Item -ItemType Directory -Path $targetRoot -Force)
[void](New-Item -ItemType Directory -Path $probeRoot -Force)
try {
    Copy-Item -LiteralPath $modulePath -Destination (Join-Path $probeRoot 'local-attestation.psm1')
    Copy-Item -LiteralPath $producerModulePath -Destination (Join-Path $probeRoot 'producer-attestation.psm1')
    $swapped = $fixture.root_anchor | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $swapped.root_id = 'live-galaxy-owner-root-v1'
    $swapped | Add-Member -NotePropertyName policy_digest -NotePropertyValue '6497cd18a4ee0286f0d566978c9225eaa168ba5d71f8fd7042a78507e1462854'
    $swapped | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $targetRoot 'owner-root-anchor.v1.json') -Encoding utf8NoBOM
    Copy-Item -LiteralPath $certificatePath -Destination (Join-Path $targetRoot 'delegated-purpose-certificate.v1.json')
    [void](New-Item -ItemType Junction -Path (Join-Path $probeRoot 'contracts') -Target $targetRoot)
    $probe = "Import-Module '$($probeRoot.Replace("'", "''"))\local-attestation.psm1' -Force; Get-OwnerAuthorityStatus | ConvertTo-Json -Compress"
    $junctionOutput = & pwsh -NoProfile -Command $probe
    Assert-True ($LASTEXITCODE -eq 0) 'Junction-swap probe did not complete.'
    $junctionStatus = $junctionOutput | ConvertFrom-Json
    Assert-Code $junctionStatus 'OWNER_ROOT_PATH_REJECTED' 'authority directory junction swap'
}
finally {
    Remove-Item -LiteralPath $junctionRoot -Recurse -Force
}

Write-Output 'PASS: held-out owner authority adversarial contract'
