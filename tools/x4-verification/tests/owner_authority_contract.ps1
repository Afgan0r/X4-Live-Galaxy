[CmdletBinding()]
param(
    [ValidateSet('root-delegation')]
    [string]$Case = 'root-delegation'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$toolRoot = Split-Path -Parent $PSScriptRoot
$modulePath = Join-Path $toolRoot 'local-attestation.psm1'
$anchorPath = Join-Path $toolRoot 'contracts/owner-root-anchor.v1.json'
$certificatePath = Join-Path $toolRoot 'contracts/delegated-purpose-certificate.v1.json'
$fixturePath = Join-Path $PSScriptRoot 'fixtures/test-owner-root-fixture.v1.json'

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Assert-Code($Result, [string]$Code, [string]$Label) {
    Assert-True ($Result.status -eq $Code) "$Label returned $($Result.status), expected $Code."
}

foreach ($path in @($modulePath, $anchorPath, $certificatePath, $fixturePath)) {
    Assert-True (Test-Path -LiteralPath $path -PathType Leaf) "Required authority artifact is missing: $path"
}

Import-Module $modulePath -Force
$status = Get-OwnerAuthorityStatus
Assert-Code $status 'OWNER_ROOT_UNCONFIGURED' 'production anchor'
Assert-True ($status.authority_ready -eq $false) 'Unconfigured production authority reported ready.'
Assert-True ($status.PSObject.Properties.Name -notcontains 'private_key') 'Production status exposed private key data.'

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

Write-Output 'PASS: owner root delegation contract'
