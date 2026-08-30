[CmdletBinding()]
param(
    [ValidateSet('root-delegation')]
    [string]$Case = 'root-delegation'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$toolRoot = Split-Path -Parent $PSScriptRoot
$modulePath = Join-Path $toolRoot 'local-attestation.psm1'
$boundedReaderPath = Join-Path $toolRoot 'bounded-file.psm1'
$anchorPath = Join-Path $toolRoot 'contracts/owner-root-anchor.v1.json'
$certificatePath = Join-Path $toolRoot 'contracts/delegated-purpose-certificate.v1.json'
$fixturePath = Join-Path $PSScriptRoot 'fixtures/test-owner-root-fixture.v1.json'
$producerModulePath = Join-Path $toolRoot 'producer-attestation.psm1'

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Assert-Code($Result, [string]$Code, [string]$Label) {
    Assert-True ($Result.status -eq $Code) "$Label returned $($Result.status), expected $Code."
}

foreach ($path in @($modulePath, $anchorPath, $certificatePath, $fixturePath)) {
    Assert-True (Test-Path -LiteralPath $path -PathType Leaf) "Required authority artifact is missing: $path"
}

Import-Module $producerModulePath -Force
Import-Module $modulePath -Force
$status = Get-OwnerAuthorityStatus
Assert-Code $status 'OWNER_ROOT_UNCONFIGURED' 'production anchor'
Assert-True ($status.authority_ready -eq $false) 'Unconfigured production authority reported ready.'
Assert-True ($status.PSObject.Properties.Name -notcontains 'private_key') 'Production status exposed private key data.'
Assert-Code (Get-OwnerOverrideDelegationStatus) 'OWNER_ROOT_UNCONFIGURED' 'production delegation'
$signingStatus = Get-OwnerOverrideSigningStatus
if ($IsWindows) {
    Assert-Code $signingStatus 'OWNER_ROOT_UNCONFIGURED' 'production signing authority'
    $silentKey = [Security.Cryptography.CngKey]::Create([Security.Cryptography.CngAlgorithm]::ECDsaP256)
    try {
        $silentSigner = [Security.Cryptography.ECDsaCng]::new($silentKey)
        try {
            $silentSignature = $silentSigner.SignData(
                [Text.Encoding]::UTF8.GetBytes('TEST-ONLY-silent-signing'),
                [Security.Cryptography.HashAlgorithmName]::SHA256
            )
            Assert-True ($silentSignature.Length -gt 0) 'Silent test key could not demonstrate unattended signing.'
        }
        finally { $silentSigner.Dispose() }
        Assert-True (-not (Test-ProductionCngSigningKeyPolicy $silentKey)) `
            'Shared producer/locator policy accepted a silent non-exportable key.'
    }
    finally { $silentKey.Dispose() }
}
else {
    Assert-Code $signingStatus 'OWNER_AUTHORITY_PLATFORM_UNSUPPORTED' 'non-Windows production signing authority'
}

$fixture = Get-Content -LiteralPath $fixturePath -Raw -Encoding utf8 | ConvertFrom-Json
$positive = Test-TestOnlyDelegatedCertificate -FixturePath $fixturePath
Assert-Code $positive 'OWNER_DELEGATION_VERIFIED' 'test-only delegation'

$wrongPurpose = $fixture | ConvertTo-Json -Depth 16 | ConvertFrom-Json
$wrongPurpose.delegated_certificate.purpose = 'candidate-producer'
Assert-Code (Test-TestOnlyDelegatedCertificateObject -Fixture $wrongPurpose) 'OWNER_DELEGATION_PURPOSE_MISMATCH' 'purpose confusion'

$rollback = $fixture | ConvertTo-Json -Depth 16 | ConvertFrom-Json
$rollback.delegated_certificate.epoch = 0
Assert-Code (Test-TestOnlyDelegatedCertificateObject -Fixture $rollback) 'OWNER_DELEGATION_EPOCH_ROLLBACK' 'epoch rollback'

$substitution = $fixture | ConvertTo-Json -Depth 16 | ConvertFrom-Json
$substitution.delegated_certificate.delegated_spki_sha256 = '0' * 64
Assert-Code (Test-TestOnlyDelegatedCertificateObject -Fixture $substitution) 'OWNER_DELEGATED_KEY_MISMATCH' 'delegated key substitution'

$command = Get-Command Get-OwnerAuthorityStatus
Assert-True ($command.Parameters.Keys -notcontains 'AnchorPath') 'Production status accepts caller anchor redirection.'
Assert-True ($command.Parameters.Keys -notcontains 'RootPath') 'Production status accepts caller root redirection.'
Assert-True ($fixture.root_anchor.root_spki_sha256 -ne 'unconfigured') 'Test fixture does not use a distinct root.'

$tempRoot = Join-Path ([IO.Path]::GetTempPath()) ("live-galaxy-root-swap-{0}" -f [guid]::NewGuid().ToString('N'))
[void](New-Item -ItemType Directory -Path (Join-Path $tempRoot 'contracts') -Force)
try {
    Copy-Item -LiteralPath $modulePath -Destination (Join-Path $tempRoot 'local-attestation.psm1')
    Copy-Item -LiteralPath $boundedReaderPath -Destination (Join-Path $tempRoot 'bounded-file.psm1')
    Copy-Item -LiteralPath $producerModulePath -Destination (Join-Path $tempRoot 'producer-attestation.psm1')
    $swapped = $fixture.root_anchor | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $swapped.root_id = 'live-galaxy-owner-root-v1'
    $swapped | Add-Member -NotePropertyName policy_digest -NotePropertyValue '6497cd18a4ee0286f0d566978c9225eaa168ba5d71f8fd7042a78507e1462854'
    $swapped | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $tempRoot 'contracts/owner-root-anchor.v1.json') -Encoding utf8NoBOM
    Copy-Item -LiteralPath $certificatePath -Destination (Join-Path $tempRoot 'contracts/delegated-purpose-certificate.v1.json')
    $probe = "Import-Module '$($tempRoot.Replace("'", "''"))\local-attestation.psm1' -Force; Get-OwnerAuthorityStatus | ConvertTo-Json -Compress"
    $swapOutput = & pwsh -NoProfile -Command $probe
    Assert-True ($LASTEXITCODE -eq 0) 'Root-swap probe did not complete.'
    $swapStatus = $swapOutput | ConvertFrom-Json
    Assert-Code $swapStatus 'OWNER_ROOT_PIN_MISMATCH' 'repository root swap'
}
finally {
    Remove-Item -LiteralPath $tempRoot -Recurse -Force
}

Write-Output 'PASS: owner root delegation contract'
