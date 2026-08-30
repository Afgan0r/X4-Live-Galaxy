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
$procedurePath = Join-Path $root 'tests/x4-candidates/05.1-candidate-run-procedure.md'
$manifestContractPath = Join-Path $root 'tools/x4-verification/contracts/candidate-build-manifest.v1.json'
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

function Get-IndependentAdapterCases {
    return @(
        @{ id = 'p051-cadence-seta'; expected = 'cadence:seta-ratio=6'; work_units = 4; good = [pscustomobject]@{ real_ms = 1000; game_ms = 6000; seta_active = $true; sample_count = 4 }; bad = [pscustomobject]@{ real_ms = 1000; game_ms = 5000; seta_active = $true; sample_count = 4 } },
        @{ id = 'p051-lifecycle-reload'; expected = 'lifecycle:single-registration'; work_units = 3; good = [pscustomobject]@{ registration_ids = @('live-galaxy-candidate'); reload_count = 1 }; bad = [pscustomobject]@{ registration_ids = @('live-galaxy-candidate', 'live-galaxy-candidate'); reload_count = 1 } },
        @{ id = 'p051-mod-stack-compatibility'; expected = 'mod-stack:declared-coexistence'; work_units = 4; good = [pscustomobject]@{ enabled_mod_ids = @('kuda-ai-tweaks', 'more-ai-economy-ships', 'add-more-sectors'); excluded_mod_ids = @('faction-enhancer') }; bad = [pscustomobject]@{ enabled_mod_ids = @('kuda-ai-tweaks', 'faction-enhancer', 'add-more-sectors'); excluded_mod_ids = @('faction-enhancer') } },
        @{ id = 'p051-native-count-fill-runtime'; expected = 'count-fill:3-of-3'; work_units = 3; good = [pscustomobject]@{ reported_count = 3; records = @('alpha', 'beta', 'gamma') }; bad = [pscustomobject]@{ reported_count = 3; records = @('alpha', 'beta') } },
        @{ id = 'p051-native-fill-completeness'; expected = 'fill:complete=3'; work_units = 3; good = [pscustomobject]@{ requested_count = 3; returned_count = 3; records = @('alpha', 'beta', 'gamma') }; bad = [pscustomobject]@{ requested_count = 3; returned_count = 2; records = @('alpha', 'beta') } },
        @{ id = 'p051-native-identity-closure'; expected = 'identity:object=station-01/owner=argon'; work_units = 4; good = [pscustomobject]@{ native_id = 'station-01'; canonical_id = 'station-01'; owner_id = 'argon'; canonical_owner_id = 'argon' }; bad = [pscustomobject]@{ native_id = 'station-01'; canonical_id = 'station-02'; owner_id = 'argon'; canonical_owner_id = 'argon' } },
        @{ id = 'p051-native-volume-envelope'; expected = 'volume:8-samples/2048-bytes'; work_units = 8; good = [pscustomobject]@{ sample_count = 8; max_samples = 16; payload_bytes = 2048; max_payload_bytes = 4096 }; bad = [pscustomobject]@{ sample_count = 17; max_samples = 16; payload_bytes = 2048; max_payload_bytes = 4096 } }
    )
}

function Test-Adapters {
    Assert-True (Test-Path -LiteralPath $adapterPath -PathType Leaf) 'Candidate adapter module is missing.'
    Assert-True (Test-Path -LiteralPath $dispatcherPath -PathType Leaf) 'Candidate dispatcher is missing.'
    Import-Module $adapterPath -Force
    $definitions = @(Get-CandidateAdapterDefinitions)
    Assert-True ((@($definitions.id | Sort-Object) -join '|') -eq ($expectedIds -join '|')) 'Adapter ID set is incomplete or unstable.'
    Assert-True (@($definitions | Where-Object { $_.classification -ne 'authenticated-local-contract' }).Count -eq 0) 'Adapters overstate or weaken evidence classification.'
    Assert-True (@($definitions | Where-Object { $_.max_work_units -lt 1 -or $_.max_work_units -gt 64 }).Count -eq 0) 'Adapter work bounds are invalid.'

    foreach ($case in Get-IndependentAdapterCases) {
        $definition = @($definitions | Where-Object id -CEQ $case.id)[0]
        $result = Invoke-CandidateAdapter -CandidateId $case.id -Fixture $case.good -MaxWorkUnits $definition.max_work_units
        Assert-True ($result.status -eq 'completed' -and $result.completeness -eq 'complete') "Adapter '$($case.id)' did not complete its typed local fixture."
        Assert-True ($result.actual_result -ceq $case.expected) "Adapter '$($case.id)' failed the independent result oracle."
        $closed = Invoke-CandidateAdapter -CandidateId $case.id -Fixture $case.bad -MaxWorkUnits $definition.max_work_units
        Assert-True ($closed.status -eq 'rejected' -and $closed.completeness -eq 'incomplete') "Adapter '$($case.id)' accepted a plausible semantic mismatch."
        $underBudget = Invoke-CandidateAdapter -CandidateId $case.id -Fixture $case.good `
            -MaxWorkUnits ($case.work_units - 1)
        Assert-True ($underBudget.status -eq 'rejected' -and
            $underBudget.completeness -eq 'incomplete' -and
            $underBudget.diagnostic_code -eq 'adapter-work-budget-exceeded') `
            "Adapter '$($case.id)' accepted a budget below its derived work."
    }

    $typeRejections = @(
        @{ id = 'p051-cadence-seta'; fixtures = @(
            [pscustomobject]@{ real_ms = '1000'; game_ms = 6000; seta_active = $true; sample_count = 4 },
            [pscustomobject]@{ real_ms = 1000.0; game_ms = 6000; seta_active = $true; sample_count = 4 },
            [pscustomobject]@{ real_ms = 1000; game_ms = 6000; seta_active = 'true'; sample_count = 4 },
            [pscustomobject]@{ real_ms = 1000; game_ms = 6000; seta_active = $true; sample_count = '4' }
        ) },
        @{ id = 'p051-lifecycle-reload'; fixtures = @(
            [pscustomobject]@{ registration_ids = 'live-galaxy-candidate'; reload_count = 1 },
            [pscustomobject]@{ registration_ids = @('live-galaxy-candidate'); reload_count = '1' },
            [pscustomobject]@{ registration_ids = @('live-galaxy-candidate'); reload_count = 1.0 }
        ) },
        @{ id = 'p051-mod-stack-compatibility'; fixtures = @(
            [pscustomobject]@{ enabled_mod_ids = 'add-more-sectors'; excluded_mod_ids = @('faction-enhancer') },
            [pscustomobject]@{ enabled_mod_ids = @('kuda-ai-tweaks', 'more-ai-economy-ships', 'add-more-sectors'); excluded_mod_ids = 'faction-enhancer' }
        ) },
        @{ id = 'p051-native-count-fill-runtime'; fixtures = @(
            [pscustomobject]@{ reported_count = '3'; records = @('alpha', 'beta', 'gamma') },
            [pscustomobject]@{ reported_count = 3.0; records = @('alpha', 'beta', 'gamma') },
            [pscustomobject]@{ reported_count = 3; records = 'abc' }
        ) },
        @{ id = 'p051-native-fill-completeness'; fixtures = @(
            [pscustomobject]@{ requested_count = '3'; returned_count = 3; records = @('alpha', 'beta', 'gamma') },
            [pscustomobject]@{ requested_count = 3; returned_count = 3.0; records = @('alpha', 'beta', 'gamma') },
            [pscustomobject]@{ requested_count = 3; returned_count = 3; records = 'abc' }
        ) },
        @{ id = 'p051-native-identity-closure'; fixtures = @(
            [pscustomobject]@{ native_id = 1; canonical_id = '1'; owner_id = 'argon'; canonical_owner_id = 'argon' }
        ) },
        @{ id = 'p051-native-volume-envelope'; fixtures = @(
            [pscustomobject]@{ sample_count = '8'; max_samples = 16; payload_bytes = 2048; max_payload_bytes = 4096 },
            [pscustomobject]@{ sample_count = 8.0; max_samples = 16; payload_bytes = 2048; max_payload_bytes = 4096 }
        ) }
    )
    foreach ($typeCase in $typeRejections) {
        $definition = @($definitions | Where-Object id -CEQ $typeCase.id)[0]
        foreach ($fixture in $typeCase.fixtures) {
            $result = Invoke-CandidateAdapter -CandidateId $typeCase.id -Fixture $fixture `
                -MaxWorkUnits $definition.max_work_units
            Assert-True ($result.status -eq 'rejected' -and $result.completeness -eq 'incomplete') `
                "Adapter '$($typeCase.id)' accepted a malformed field type."
        }
    }

    $alternateIdentity = [pscustomobject]@{
        native_id = 'ship-99'; canonical_id = 'ship-99'
        owner_id = 'boron'; canonical_owner_id = 'boron'
    }
    $identityResult = Invoke-CandidateAdapter -CandidateId 'p051-native-identity-closure' `
        -Fixture $alternateIdentity -MaxWorkUnits 16
    $identityExpected = "identity:object=$($alternateIdentity.native_id)/owner=$($alternateIdentity.owner_id)"
    Assert-True ($identityResult.actual_result -ceq $identityExpected) `
        'Alternate valid identity did not derive its result from the fixture.'
    Assert-True (($identityResult.observations -join '|') -ceq 'object=ship-99|owner=boron') `
        'Alternate valid identity did not derive its observations from the fixture.'
    Assert-True ($identityResult.work_units -eq 4) `
        'Alternate valid identity reported unexpected work units.'
    $identityBudgetRejected = Invoke-CandidateAdapter `
        -CandidateId 'p051-native-identity-closure' `
        -Fixture $alternateIdentity -MaxWorkUnits 3
    Assert-True ($identityBudgetRejected.status -eq 'rejected' -and
        $identityBudgetRejected.completeness -eq 'incomplete' -and
        $identityBudgetRejected.diagnostic_code -eq 'adapter-work-budget-exceeded') `
        'Identity adapter accepted a budget one unit below derived work.'

    $alternateVolume = [pscustomobject]@{
        sample_count = 7; max_samples = 16
        payload_bytes = 1000; max_payload_bytes = 4096
    }
    $volumeResult = Invoke-CandidateAdapter -CandidateId 'p051-native-volume-envelope' `
        -Fixture $alternateVolume -MaxWorkUnits 16
    $volumeExpected = "volume:$($alternateVolume.sample_count)-samples/$($alternateVolume.payload_bytes)-bytes"
    Assert-True ($volumeResult.actual_result -ceq $volumeExpected) `
        'Alternate valid volume did not derive its result from the fixture.'
    Assert-True (($volumeResult.observations -join '|') -ceq 'samples=7/16|bytes=1000/4096') `
        'Alternate valid volume did not derive its observations from the fixture.'
    Assert-True ($volumeResult.work_units -eq $alternateVolume.sample_count) `
        'Alternate valid volume did not derive work units from the fixture.'
    $volumeBudgetRejected = Invoke-CandidateAdapter `
        -CandidateId 'p051-native-volume-envelope' `
        -Fixture $alternateVolume -MaxWorkUnits 6
    Assert-True ($volumeBudgetRejected.status -eq 'rejected' -and
        $volumeBudgetRejected.completeness -eq 'incomplete' -and
        $volumeBudgetRejected.diagnostic_code -eq 'adapter-work-budget-exceeded') `
        'Volume adapter accepted a budget one unit below derived work.'

    $temp = Join-Path ([IO.Path]::GetTempPath()) ("live-galaxy-plan08-runtime-" + [guid]::NewGuid().ToString('N'))
    $buildRoot = Join-Path $temp 'builds'
    $outputRoot = Join-Path $temp 'output'
    $null = New-Item -ItemType Directory -Path $outputRoot -Force
    if ($IsWindows) {
        $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        $null = & icacls.exe $outputRoot /inheritance:r /grant:r "*$sid`:(OI)(CI)F"
        Assert-True ($LASTEXITCODE -eq 0) 'Unable to protect dispatcher output fixture.'
    }
    try {
        $build = Invoke-JsonCommand $builderPath @('-BuildRoot', $buildRoot, '-MatrixPath', $matrixPath)
        Assert-True ($build.verdict -in @('generated', 'pass')) 'Candidate builder failed before dispatcher verification.'
        $attackGroupRoot = Join-Path $buildRoot 'p051-build-lifecycle'
        $attackManifestPath = Join-Path $attackGroupRoot 'manifest/build-manifest.v1.json'
        [byte[]]$originalManifestBytes = [IO.File]::ReadAllBytes($attackManifestPath)
        $manifestContract = Get-Content -LiteralPath $manifestContractPath -Raw | ConvertFrom-Json
        $boundCases = @(
            [pscustomobject]@{
                Name = 'generated-count-max-plus-one'; Code = 'GENERATED_BOUNDS_EXCEEDED'
                Mutate = { param($manifest)
                    $extra = $manifest.generated_files[0]
                    $manifest.generated_files = @($manifest.generated_files) + @(
                        $extra
                    ) * ($manifestContract.bounds.max_generated_files + 1 - @($manifest.generated_files).Count)
                }
            },
            [pscustomobject]@{
                Name = 'generated-file-bytes-max-plus-one'; Code = 'GENERATED_FILE_BYTES_EXCEEDED'
                Mutate = { param($manifest)
                    $manifest.generated_files[0].bytes = $manifestContract.bounds.max_generated_file_bytes + 1
                }
            },
            [pscustomobject]@{
                Name = 'generated-total-bytes-max-plus-one'; Code = 'GENERATED_BOUNDS_EXCEEDED'
                Mutate = { param($manifest)
                    foreach ($file in @($manifest.generated_files)) {
                        $file.bytes = $manifestContract.bounds.max_generated_file_bytes
                    }
                }
            }
        )
        foreach ($boundCase in $boundCases) {
            $manifest = [Text.Encoding]::UTF8.GetString($originalManifestBytes) |
                ConvertFrom-Json -Depth 64 -DateKind String
            & $boundCase.Mutate $manifest
            [IO.File]::WriteAllText($attackManifestPath,
                ($manifest | ConvertTo-Json -Compress -Depth 64), [Text.UTF8Encoding]::new($false))
            $attackOutput = Join-Path $outputRoot "$($boundCase.Name).jsonl"
            $rejected = Invoke-JsonCommand $dispatcherPath @(
                '-GroupRoot', $attackGroupRoot, '-OutputPath', $attackOutput
            ) 1
            Assert-True ($rejected.reason_code -eq $boundCase.Code) `
                "Dispatcher returned '$($rejected.reason_code)' for '$($boundCase.Name)'."
            Assert-True (-not (Test-Path -LiteralPath $attackOutput)) `
                "Rejected dispatcher bound '$($boundCase.Name)' wrote evidence."
        }
        [IO.File]::WriteAllBytes($attackManifestPath, $originalManifestBytes)
        $expectedCommand = 'pwsh -NoProfile -File tools/x4-verification/run-candidate-package.ps1 -GroupRoot $GroupRoot -OutputPath $PrivateJsonlPath'
        Assert-True ((Get-Content -LiteralPath $procedurePath -Raw).Contains($expectedCommand)) `
            'Human handoff does not name the repository-fixed dispatcher call chain.'
        $observed = @()
        foreach ($group in @('p051-build-lifecycle', 'p051-build-read-only-shared')) {
            $groupRoot = Join-Path $buildRoot $group
            Assert-True (-not (Test-Path -LiteralPath (
                Join-Path $groupRoot 'tools/x4-verification/run-candidate-package.ps1'
            ))) 'Generated package exposes a misleading package-local dispatcher.'
            $output = Join-Path $outputRoot "$group.jsonl"
            $run = Invoke-JsonCommand $dispatcherPath @('-GroupRoot', $groupRoot, '-OutputPath', $output)
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
