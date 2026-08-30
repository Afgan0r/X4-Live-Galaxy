[CmdletBinding()]
param(
    [ValidateSet('adapters', 'all')]
    [string]$Case = 'all'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$builderPath = Join-Path $root 'tools/x4-verification/build-candidate-extension.ps1'
$dispatcherPath = Join-Path $root 'tools/x4-verification/run-candidate-package.ps1'
$adapterPath = Join-Path $root 'tools/x4-verification/candidate-adapters.psm1'
$matrixPath = Join-Path $root 'tests/x4-candidates/phase-05.1-candidates.v1.json'
$expectedIds = @(
    'p051-cadence-seta',
    'p051-lifecycle-reload',
    'p051-mod-stack-compatibility',
    'p051-native-count-fill-runtime',
    'p051-native-fill-completeness',
    'p051-native-identity-closure',
    'p051-native-volume-envelope'
)

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Invoke-JsonCommand([string]$File, [string[]]$Arguments, [int]$ExpectedExit = 0) {
    $text = & pwsh -NoProfile -File $File @Arguments 2>&1 | Out-String
    Assert-True ($LASTEXITCODE -eq $ExpectedExit) "Unexpected exit $LASTEXITCODE from $File`: $text"
    try { return $text.Trim() | ConvertFrom-Json -DateKind String }
    catch { throw "Command did not emit JSON: $text" }
}

function Test-Adapters {
    Assert-True (Test-Path -LiteralPath $adapterPath -PathType Leaf) 'Candidate adapter module is missing.'
    Assert-True (Test-Path -LiteralPath $dispatcherPath -PathType Leaf) 'Candidate dispatcher is missing.'

    Import-Module $adapterPath -Force
    $definitions = @(Get-CandidateAdapterDefinitions)
    Assert-True ((@($definitions.id | Sort-Object) -join '|') -eq ($expectedIds -join '|')) 'Adapter ID set is incomplete or unstable.'
    Assert-True (@($definitions | Where-Object { $_.classification -ne 'authenticated-local-contract' }).Count -eq 0) 'Adapters overstate or weaken evidence classification.'
    Assert-True (@($definitions | Where-Object { $_.max_work_units -lt 1 -or $_.max_work_units -gt 64 }).Count -eq 0) 'Adapter work bounds are invalid.'

    foreach ($definition in $definitions) {
        $result = Invoke-CandidateAdapter -CandidateId $definition.id -ExpectedResult "expected-$($definition.id)" -MaxWorkUnits $definition.max_work_units
        Assert-True ($result.status -eq 'completed' -and $result.completeness -eq 'complete') "Adapter '$($definition.id)' did not complete its local fixture."
        Assert-True ($result.actual_result -eq "expected-$($definition.id)") "Adapter '$($definition.id)' changed the expected effect identity."
    }
    foreach ($attack in @(
        @{ id = 'p051-native-fill-completeness'; fixture = 'partial' },
        @{ id = 'p051-native-identity-closure'; fixture = 'foreign-owner' },
        @{ id = 'p051-native-volume-envelope'; fixture = 'bound-exceeded' },
        @{ id = 'p051-cadence-seta'; fixture = 'timeout' },
        @{ id = 'p051-lifecycle-reload'; fixture = 'duplicate-registration' },
        @{ id = 'p051-mod-stack-compatibility'; fixture = 'excluded-suite' },
        @{ id = 'p051-native-count-fill-runtime'; fixture = 'malformed-count' }
    )) {
        $closed = Invoke-CandidateAdapter -CandidateId $attack.id -ExpectedResult 'never-pass' -MaxWorkUnits 8 -Fixture $attack.fixture
        Assert-True ($closed.status -eq 'rejected' -and $closed.completeness -eq 'incomplete') "Adapter attack '$($attack.fixture)' did not fail closed."
    }

    $temp = Join-Path ([IO.Path]::GetTempPath()) ("live-galaxy-plan08-runtime-" + [guid]::NewGuid().ToString('N'))
    $buildRoot = Join-Path $temp 'builds'
    $outputRoot = Join-Path $temp 'output'
    $null = New-Item -ItemType Directory -Path $outputRoot -Force
    try {
        $build = Invoke-JsonCommand $builderPath @('-BuildRoot', $buildRoot, '-MatrixPath', $matrixPath)
        Assert-True ($build.verdict -in @('generated', 'pass')) 'Candidate builder failed before dispatcher verification.'
        $observed = @()
        foreach ($group in @('p051-build-lifecycle', 'p051-build-read-only-shared')) {
            $output = Join-Path $outputRoot "$group.jsonl"
            $run = Invoke-JsonCommand $dispatcherPath @('-GroupRoot', (Join-Path $buildRoot $group), '-OutputPath', $output)
            Assert-True ($run.local_process_ready -eq $true) "Dispatcher did not prove local readiness for '$group'."
            Assert-True ($run.evidence_classification -eq 'authenticated-local-contract') 'Dispatcher made a non-local evidence claim.'
            Assert-True ($run.retainable -eq $false -and $run.attestation_status -eq 'PRODUCER_ATTESTATION_UNCONFIGURED') 'Unprovisioned production authority did not remain explicitly non-retainable.'
            Assert-True (Test-Path -LiteralPath $output -PathType Leaf) 'Dispatcher did not atomically publish JSONL.'
            foreach ($line in @(Get-Content -LiteralPath $output)) {
                $row = $line | ConvertFrom-Json -DateKind String
                $observed += [string]$row.candidate_id
                Assert-True ($row.execution_verdict -eq 'pass' -and $row.contract_verdict -eq 'pass' -and $row.effect_verdict -eq 'pass') 'Dispatcher collapsed or failed a verdict axis.'
                Assert-True ($row.evidence_classification -eq 'authenticated-local-contract') 'JSONL row overstates X4 observation.'
            }
        }
        Assert-True ((@($observed | Sort-Object) -join '|') -eq ($expectedIds -join '|')) 'Dispatcher did not run all seven candidates exactly once.'
    }
    finally {
        if (Test-Path -LiteralPath $temp) { Remove-Item -LiteralPath $temp -Recurse -Force }
    }
    Write-Output 'candidate-package-runtime: adapters PASS'
}

if ($Case -in @('adapters', 'all')) { Test-Adapters }
