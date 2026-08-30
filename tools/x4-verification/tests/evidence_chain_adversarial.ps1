[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$toolRoot = Split-Path -Parent $PSScriptRoot
$repositoryRoot = Split-Path -Parent (Split-Path -Parent $toolRoot)
$admissionContractPath = Join-Path $PSScriptRoot 'admission_contract.ps1'
$aggregatePath = Join-Path $repositoryRoot 'extensions/live_galaxy/tests/run_contracts.ps1'

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

$output = @(& pwsh -NoProfile -File $admissionContractPath -Case evidence-chain 2>&1)
Assert-True ($LASTEXITCODE -eq 0) "Evidence-chain contract failed: $($output -join ' | ')"
$joined = $output -join "`n"
foreach ($marker in @(
        'PASS: hand-authored evidence-chain rejection contract',
        'PASS: verified locator evidence-chain contract',
        'PASS: evidence-chain forgery rejection contract'
    )) {
    Assert-True ($joined.Contains($marker)) "Evidence-chain contract omitted marker: $marker"
}

$aggregateSource = Get-Content -LiteralPath $aggregatePath -Raw
Assert-True ($aggregateSource.Contains('evidence_chain_adversarial.ps1')) 'Aggregate does not run the held-out chain suite.'
Assert-True ($aggregateSource.Contains('held-out evidence-chain adversarial contract')) 'Aggregate does not require the held-out marker.'

Write-Output 'PASS: held-out evidence-chain adversarial contract'
