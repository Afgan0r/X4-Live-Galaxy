[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$toolRoot = Split-Path -Parent $PSScriptRoot
$repositoryRoot = Split-Path -Parent (Split-Path -Parent $toolRoot)
$retentionContractPath = Join-Path $PSScriptRoot 'evidence_retention_contract.ps1'
$retentionPath = Join-Path $toolRoot 'retain-evidence.ps1'
$admissionPath = Join-Path $toolRoot 'x4-admission.ps1'
$dossierPath = Join-Path $toolRoot 'contracts/phase-05.1-dossier.v1.json'
$registryPath = Join-Path $toolRoot 'contracts/known-failures.v1.json'
$coveragePath = Join-Path $toolRoot 'contracts/coverage.v1.json'
$fixturePath = Join-Path $toolRoot 'fixtures/negative-fixtures.v1.json'
$matrixPath = Join-Path $repositoryRoot 'tests/x4-candidates/phase-05.1-candidates.v1.json'
$pendingLedgerPath = Join-Path $repositoryRoot 'tests/x4-candidates/phase-05.1-candidate-ledger.v1.json'

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Invoke-Admission([string[]]$ExtraArguments) {
    $output = @(& pwsh -NoProfile -File $admissionPath `
        -DossierPath $dossierPath -RegistryPath $registryPath `
        -CoveragePath $coveragePath -FixturePath $fixturePath @ExtraArguments 2>&1)
    Assert-True ($LASTEXITCODE -ne 0) 'Held-out forged admission unexpectedly succeeded.'
    $json = @($output | Where-Object { $_.ToString().TrimStart().StartsWith('{') })[-1] |
        ConvertFrom-Json -Depth 16 -DateKind String
    return $json
}

$chainOutput = @(& pwsh -NoProfile -File $retentionContractPath -Case retention-admission 2>&1)
Assert-True ($LASTEXITCODE -eq 0) "Production-serialization chain failed: $($chainOutput -join ' | ')"

$scratch = Join-Path ([IO.Path]::GetTempPath()) (
    'live-galaxy-held-out-chain-' + [guid]::NewGuid().ToString('N')
)
$null = New-Item -ItemType Directory -Path $scratch
if ($IsWindows) {
    $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    $null = & icacls.exe $scratch /inheritance:r /grant:r "*$sid`:(OI)(CI)F"
    Assert-True ($LASTEXITCODE -eq 0) 'Unable to protect held-out chain fixture.'
}
try {
    $forgedLedgerPath = Join-Path $scratch 'forged-ledger.json'
    $forgedLedger = Get-Content -LiteralPath $pendingLedgerPath -Raw | ConvertFrom-Json -Depth 32
    $forgedLedger.status = 'completed'
    $forgedLedger.candidates[0].execution_verdict = 'pass'
    [IO.File]::WriteAllText(
        $forgedLedgerPath, ($forgedLedger | ConvertTo-Json -Depth 32),
        [Text.UTF8Encoding]::new($false)
    )
    $ledgerResult = Invoke-Admission @(
        '-SanitizedLedgerPath', $forgedLedgerPath,
        '-PendingLedgerPath', $pendingLedgerPath,
        '-CandidateMatrixPath', $matrixPath
    )
    Assert-True ($ledgerResult.verdict -ne 'admissible') `
        'Held-out hand-authored ledger did not fail closed.'
    Assert-True (@($ledgerResult.reason_codes).Count -gt 0) `
        'Held-out hand-authored ledger lacked a structured rejection reason.'

    $forgedLocatorPath = Join-Path $scratch 'forged-locator.json'
    [IO.File]::WriteAllText(
        $forgedLocatorPath,
        '{"schema_version":"private-evidence-locator.v1","signature_base64":"forged"}',
        [Text.UTF8Encoding]::new($false)
    )
    $locatorOutput = @(& pwsh -NoProfile -File $retentionPath `
        -VerifyLocatorPath $forgedLocatorPath 2>&1)
    Assert-True ($LASTEXITCODE -ne 0) 'Held-out forged locator unexpectedly verified.'
    $locatorResult = @($locatorOutput | Where-Object {
        $_.ToString().TrimStart().StartsWith('{')
    })[-1] | ConvertFrom-Json -Depth 8
    Assert-True ($locatorResult.verdict -eq 'rejected') 'Forged locator lacked structured rejection.'
    Assert-True (@(Get-ChildItem -LiteralPath $scratch -Force).Count -eq 2) `
        'Held-out chain rejection created an unexpected durable artifact.'

    $targetRoot = Join-Path $scratch 'locator-target'
    $null = New-Item -ItemType Directory -Path $targetRoot
    Copy-Item -LiteralPath $forgedLocatorPath -Destination (Join-Path $targetRoot 'locator.json')
    $reparsePath = Join-Path $scratch 'locator-link.json'
    $itemType = if ($IsWindows) { 'SymbolicLink' } else { 'SymbolicLink' }
    try {
        $null = New-Item -ItemType $itemType -Path $reparsePath `
            -Target (Join-Path $targetRoot 'locator.json')
        $reparseOutput = @(& pwsh -NoProfile -File $retentionPath `
            -VerifyLocatorPath $reparsePath 2>&1)
        Assert-True ($LASTEXITCODE -ne 0) 'Held-out locator reparse unexpectedly verified.'
    }
    catch [UnauthorizedAccessException] {
        # Windows developer mode may disable unprivileged file symlinks. The
        # directory-junction cases remain active in the focused contracts.
    }
}
finally {
    if (Test-Path -LiteralPath $scratch) { Remove-Item -LiteralPath $scratch -Recurse -Force }
}

Write-Output 'PASS: held-out evidence-chain adversarial contract'
