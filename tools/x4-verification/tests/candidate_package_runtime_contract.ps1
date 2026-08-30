[CmdletBinding()]
param(
    [ValidateSet('adapters', 'reuse-contract', 'all')]
    [string]$Case = 'all',
    [string]$PreparedBuildRoot,
    [string]$PreparedBuildKey
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

function Get-Digest([string]$Path) {
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Assert-OwnerOnly([string]$Path) {
    if ($IsWindows) {
        $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        $acl = Get-Acl -LiteralPath $Path
        Assert-True ($acl.GetOwner([Security.Principal.SecurityIdentifier]).Value -eq $sid) `
            'PREPARED_BUILD_OWNER_MISMATCH'
        foreach ($rule in @($acl.Access | Where-Object AccessControlType -eq 'Allow')) {
            $ruleSid = $rule.IdentityReference.Translate(
                [Security.Principal.SecurityIdentifier]
            ).Value
            Assert-True ($ruleSid -eq $sid) 'PREPARED_BUILD_NOT_OWNER_ONLY'
        }
        return
    }
    $mode = [IO.File]::GetUnixFileMode($Path)
    $forbidden = [IO.UnixFileMode]::GroupRead -bor [IO.UnixFileMode]::GroupWrite -bor `
        [IO.UnixFileMode]::GroupExecute -bor [IO.UnixFileMode]::OtherRead -bor `
        [IO.UnixFileMode]::OtherWrite -bor [IO.UnixFileMode]::OtherExecute
    Assert-True (($mode -band $forbidden) -eq 0) 'PREPARED_BUILD_NOT_OWNER_ONLY'
}

function Get-PreparedKey([string]$BuildRoot) {
    $matrixDigest = Get-Digest $matrixPath
    $material = [Collections.Generic.List[string]]::new()
    $material.Add("matrix=$matrixDigest")
    foreach ($sourcePath in @(
        'tools/x4-verification/build-candidate-extension.ps1',
        'tools/x4-verification/contracts/candidate-build-manifest.v1.json',
        'tools/x4-verification/templates/candidate-content.xml',
        'tools/x4-verification/templates/candidate-entry.lua',
        'tools/x4-verification/templates/candidate-ui.xml',
        'tests/x4-candidates/lua/live_galaxy_candidate_runner.lua'
    )) {
        $material.Add("source/$sourcePath=$(Get-Digest (Join-Path $root $sourcePath))")
    }
    $componentBindings = [ordered]@{
        dossier_digest = 'tools/x4-verification/contracts/phase-05.1-dossier.v1.json'
        registry_digest = 'tools/x4-verification/contracts/known-failures.v1.json'
        coverage_digest = 'tools/x4-verification/contracts/coverage.v1.json'
        runtime_evidence_schema_digest = 'tools/x4-verification/contracts/runtime-evidence.v1.json'
        owner_root_anchor_digest = 'tools/x4-verification/contracts/owner-root-anchor.v1.json'
        dispatcher_digest = 'tools/x4-verification/run-candidate-package.ps1'
        adapter_digest = 'tools/x4-verification/candidate-adapters.psm1'
        attestation_module_digest = 'tools/x4-verification/producer-attestation.psm1'
        bounded_reader_digest = 'tools/x4-verification/bounded-file.psm1'
        worker_digest = 'tools/x4-verification/isolation/candidate-worker.ps1'
        launcher_digest = 'tools/x4-verification/isolation/invoke-candidate-worker.ps1'
        worker_protocol_digest = 'tools/x4-verification/contracts/candidate-worker-protocol.v1.json'
    }
    $groups = @(Get-ChildItem -LiteralPath $BuildRoot -Directory | Sort-Object Name)
    Assert-True ($groups.Count -gt 0 -and $groups.Count -le 16) 'PREPARED_BUILD_GROUPS_INVALID'
    foreach ($group in $groups) {
        Assert-True (($group.Attributes -band [IO.FileAttributes]::ReparsePoint) -eq 0) `
            'PREPARED_BUILD_REPARSE_REJECTED'
        $manifestPath = Join-Path $group.FullName 'manifest/build-manifest.v1.json'
        Assert-True (Test-Path -LiteralPath $manifestPath -PathType Leaf) `
            'PREPARED_BUILD_MANIFEST_MISSING'
        $manifestItem = Get-Item -LiteralPath $manifestPath -Force
        Assert-True ($manifestItem.Length -le 262144 -and
            ($manifestItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -eq 0) `
            'PREPARED_BUILD_MANIFEST_INVALID'
        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json -Depth 64
        Assert-True ($manifest.matrix_digest -eq $matrixDigest) 'PREPARED_BUILD_SOURCE_MISMATCH'
        foreach ($binding in $componentBindings.GetEnumerator()) {
            Assert-True ($manifest.($binding.Key) -eq (Get-Digest (Join-Path $root $binding.Value))) `
                'PREPARED_BUILD_SOURCE_MISMATCH'
        }
        $material.Add("manifest/$($group.Name)=$(Get-Digest $manifestPath)")
        $generatedFiles = @($manifest.generated_files | Sort-Object path)
        Assert-True ($generatedFiles.Count -gt 0 -and $generatedFiles.Count -le 16) `
            'PREPARED_BUILD_FILES_INVALID'
        [long]$totalBytes = 0
        foreach ($generated in $generatedFiles) {
            $logicalPath = [string]$generated.path
            Assert-True ($logicalPath -match '^[a-zA-Z0-9._/-]+$' -and
                @($logicalPath -split '[\\/]+') -notcontains '..') `
                'PREPARED_BUILD_PATH_INVALID'
            $generatedPath = [IO.Path]::GetFullPath((Join-Path $group.FullName $logicalPath))
            Assert-True ($generatedPath.StartsWith(
                $group.FullName.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar,
                [StringComparison]::OrdinalIgnoreCase
            )) 'PREPARED_BUILD_PATH_INVALID'
            Assert-True (Test-Path -LiteralPath $generatedPath -PathType Leaf) `
                'PREPARED_BUILD_FILE_MISSING'
            $item = Get-Item -LiteralPath $generatedPath
            Assert-True ($item.Length -le 65536 -and
                ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -eq 0) `
                'PREPARED_BUILD_FILE_INVALID'
            $totalBytes += $item.Length
            $digest = Get-Digest $generatedPath
            Assert-True ($digest -eq $generated.sha256 -and $item.Length -eq $generated.bytes) `
                'PREPARED_BUILD_DIGEST_MISMATCH'
            $material.Add("$($group.Name)/$($generated.path)=$digest")
        }
        Assert-True ($totalBytes -le 524288) 'PREPARED_BUILD_FILES_INVALID'
    }
    return [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData(
        [Text.Encoding]::UTF8.GetBytes(($material -join "`n"))
    )).ToLowerInvariant()
}

function Assert-PreparedBuild([string]$BuildRoot, [string]$BuildKey) {
    Assert-True ($BuildKey -match '^[a-f0-9]{64}$') 'PREPARED_BUILD_KEY_INVALID'
    Assert-True (Test-Path -LiteralPath $BuildRoot -PathType Container) `
        'PREPARED_BUILD_MISSING'
    $full = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $BuildRoot).Path)
    $temp = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    Assert-True ($full.StartsWith($temp, [StringComparison]::OrdinalIgnoreCase)) `
        'PREPARED_BUILD_OUTSIDE_TEMP'
    $relative = [IO.Path]::GetRelativePath($temp, $full)
    $current = $temp
    foreach ($segment in @($relative -split '[\\/]+')) {
        $current = Join-Path $current $segment
        $item = Get-Item -LiteralPath $current -Force
        Assert-True (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -eq 0) `
            'PREPARED_BUILD_REPARSE_REJECTED'
    }
    Assert-OwnerOnly $full
    Assert-True ((Split-Path -Leaf $full) -eq $BuildKey) 'PREPARED_BUILD_IDENTITY_MISMATCH'
    Assert-True ((Get-PreparedKey $full) -eq $BuildKey) 'PREPARED_BUILD_KEY_MISMATCH'
}

function Copy-PreparedBuild([string]$Destination) {
    Assert-PreparedBuild $PreparedBuildRoot $PreparedBuildKey
    Copy-Item -LiteralPath $PreparedBuildRoot -Destination $Destination -Recurse
    if ($IsWindows) {
        $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        $null = & icacls.exe $Destination /inheritance:r /grant:r "*$sid`:(OI)(CI)F"
        Assert-True ($LASTEXITCODE -eq 0) 'PREPARED_BUILD_PERMISSION_FAILED'
        foreach ($item in @(Get-ChildItem -LiteralPath $Destination -Recurse -Force)) {
            $grant = if ($item.PSIsContainer) { "*$sid`:(OI)(CI)F" } else { "*$sid`:F" }
            $null = & icacls.exe $item.FullName /inheritance:r /grant:r $grant
            Assert-True ($LASTEXITCODE -eq 0) 'PREPARED_BUILD_PERMISSION_FAILED'
        }
    }
    else {
        [IO.File]::SetUnixFileMode(
            $Destination,
            [IO.UnixFileMode]::UserRead -bor [IO.UnixFileMode]::UserWrite -bor
                [IO.UnixFileMode]::UserExecute
        )
        foreach ($item in @(Get-ChildItem -LiteralPath $Destination -Recurse -Force)) {
            [IO.File]::SetUnixFileMode(
                $item.FullName,
                $(if ($item.PSIsContainer) {
                    [IO.UnixFileMode]::UserRead -bor [IO.UnixFileMode]::UserWrite -bor
                        [IO.UnixFileMode]::UserExecute
                } else {
                    [IO.UnixFileMode]::UserRead -bor [IO.UnixFileMode]::UserWrite
                })
            )
        }
    }
    Assert-PreparedBuild $Destination $PreparedBuildKey
    Assert-True ((Get-PreparedKey $PreparedBuildRoot) -eq $PreparedBuildKey) `
        'PREPARED_BUILD_SOURCE_CHANGED'
}

$hasPreparedBuild = -not [string]::IsNullOrWhiteSpace($PreparedBuildRoot) -or
    -not [string]::IsNullOrWhiteSpace($PreparedBuildKey)
Assert-True (-not $hasPreparedBuild -or (
    -not [string]::IsNullOrWhiteSpace($PreparedBuildRoot) -and
    -not [string]::IsNullOrWhiteSpace($PreparedBuildKey)
)) 'PREPARED_BUILD_PARAMETERS_INCOMPLETE'

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
    $buildRoot = Join-Path $temp $(if ($hasPreparedBuild) { $PreparedBuildKey } else { 'builds' })
    $outputRoot = Join-Path $temp 'output'
    $null = New-Item -ItemType Directory -Path $outputRoot -Force
    if ($IsWindows) {
        $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        $null = & icacls.exe $outputRoot /inheritance:r /grant:r "*$sid`:(OI)(CI)F"
        Assert-True ($LASTEXITCODE -eq 0) 'Unable to protect dispatcher output fixture.'
    }
    try {
        if ($hasPreparedBuild) {
            Copy-PreparedBuild $buildRoot
        }
        else {
            $build = Invoke-JsonCommand $builderPath @('-BuildRoot', $buildRoot, '-MatrixPath', $matrixPath)
            Assert-True ($build.verdict -in @('generated', 'pass')) 'Candidate builder failed before dispatcher verification.'
        }
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
        if ($hasPreparedBuild) { Assert-PreparedBuild $PreparedBuildRoot $PreparedBuildKey }
    }
    Write-Output 'candidate-package-runtime: adapters PASS'
}

if ($Case -eq 'reuse-contract') {
    Assert-True $hasPreparedBuild 'PREPARED_BUILD_REQUIRED'
    $probeRoot = Join-Path ([IO.Path]::GetTempPath()) `
        ('live-galaxy-runtime-reuse-' + [guid]::NewGuid().ToString('N'))
    $null = New-Item -ItemType Directory -Path $probeRoot
    try {
        $clone = Join-Path $probeRoot $PreparedBuildKey
        Copy-PreparedBuild $clone
        $generated = Get-ChildItem -LiteralPath $clone -Recurse -File |
            Where-Object FullName -notmatch 'build-manifest\.v1\.json$' |
            Select-Object -First 1
        [IO.File]::AppendAllText($generated.FullName, "`nchanged-clone")
        $changedRejected = $false
        try { Assert-PreparedBuild $clone $PreparedBuildKey }
        catch { $changedRejected = $_.Exception.Message -eq 'PREPARED_BUILD_DIGEST_MISMATCH' }
        Assert-True $changedRejected 'PREPARED_BUILD_CHANGED_CLONE_ACCEPTED'

        $outsideRejected = $false
        try { Assert-PreparedBuild $root $PreparedBuildKey }
        catch { $outsideRejected = $_.Exception.Message -eq 'PREPARED_BUILD_OUTSIDE_TEMP' }
        Assert-True $outsideRejected 'PREPARED_BUILD_OUTSIDE_TEMP_ACCEPTED'

        $target = Join-Path $probeRoot 'reparse-target'
        $link = Join-Path $probeRoot 'reparse-root'
        Copy-Item -LiteralPath $PreparedBuildRoot -Destination $target -Recurse
        $itemType = if ($IsWindows) { 'Junction' } else { 'SymbolicLink' }
        $null = New-Item -ItemType $itemType -Path $link -Target $target
        $reparseRejected = $false
        try { Assert-PreparedBuild $link $PreparedBuildKey }
        catch { $reparseRejected = $_.Exception.Message -eq 'PREPARED_BUILD_REPARSE_REJECTED' }
        Assert-True $reparseRejected 'PREPARED_BUILD_REPARSE_ACCEPTED'
        Assert-PreparedBuild $PreparedBuildRoot $PreparedBuildKey
        $consumerOutput = @(& pwsh -NoProfile -File $PSCommandPath -Case adapters `
            -PreparedBuildRoot $PreparedBuildRoot -PreparedBuildKey $PreparedBuildKey)
        Assert-True ($LASTEXITCODE -eq 0 -and
            $consumerOutput -contains 'candidate-package-runtime: adapters PASS') `
            'PREPARED_BUILD_RUNTIME_CONSUMER_FAILED'
        Write-Output 'PASS: prepared-build runtime reuse contract'
    }
    finally {
        if (Test-Path -LiteralPath $probeRoot) { Remove-Item -LiteralPath $probeRoot -Recurse -Force }
    }
    exit 0
}

if ($Case -in @('adapters', 'all')) { Test-Adapters }
