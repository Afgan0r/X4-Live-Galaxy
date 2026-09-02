param(
    [ValidateSet('all', 'x4_discovery', 'component_discovery', 'component_discovery_guard', 'x4-admission', 'x4-package-conformance', 'x4-candidate-runner', 'x4-verification-reuse-contract', 'x4-verification-fast', 'x4-verification')]
    [string]$Suite = 'all',
    [string]$LuaPath
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
if ($Suite -eq 'x4-package-conformance') {
    & pwsh -NoProfile -File (Join-Path $PSScriptRoot 'package_conformance_contract.ps1')
    if ($LASTEXITCODE -ne 0) { throw 'Product package conformance contract failed.' }
    exit 0
}
$admissionContract = Join-Path $root 'tools/x4-verification/tests/admission_contract.ps1'
$packageConformanceContract = Join-Path $root 'tools/x4-verification/tests/package_conformance_contract.ps1'
$candidateBuildContract = Join-Path $root 'tools/x4-verification/tests/candidate_build_contract.ps1'
$evidenceRetentionContract = Join-Path $root 'tools/x4-verification/tests/evidence_retention_contract.ps1'
$candidateIsolationContract = Join-Path $root 'tools/x4-verification/tests/candidate_isolation_contract.ps1'
$candidatePackageRuntimeContract = Join-Path $root 'tools/x4-verification/tests/candidate_package_runtime_contract.ps1'
$candidatePackageAdversarial = Join-Path $root 'tools/x4-verification/tests/candidate_package_adversarial.ps1'
$ownerAuthorityContract = Join-Path $root 'tools/x4-verification/tests/owner_authority_contract.ps1'
$ownerAuthorityAdversarial = Join-Path $root 'tools/x4-verification/tests/owner_authority_adversarial.ps1'
$componentDiscoveryGuard = Join-Path $root 'scripts/component_discovery_package_guard.ps1'
$componentDiscoveryProductionPaths = @(
    'extensions/live_galaxy/lua/live_galaxy_component_discovery.lua',
    'extensions/live_galaxy/lua/live_galaxy_x4_discovery.lua',
    'extensions/live_galaxy/lua/live_galaxy_runtime.lua',
    'extensions/live_galaxy/lua/live_galaxy_telemetry.lua'
)
$evidenceChainAdversarial = Join-Path $root 'tools/x4-verification/tests/evidence_chain_adversarial.ps1'
$candidateBuilder = Join-Path $root 'tools/x4-verification/build-candidate-extension.ps1'
$candidateMatrix = Join-Path $root 'tests/x4-candidates/phase-05.1-candidates.v1.json'
$script:preparedInvocationRoot = $null
$script:preparedBuildRoot = $null
$script:preparedBuildKey = $null

function Get-FileSha256([string]$Path) {
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Set-OwnerOnlyDirectory([string]$Path) {
    if ($IsWindows) {
        $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        $null = & icacls.exe $Path /inheritance:r /grant:r "*$sid`:(OI)(CI)F"
        if ($LASTEXITCODE -ne 0) { throw 'PREPARED_BUILD_PERMISSION_FAILED' }
        return
    }
    [IO.File]::SetUnixFileMode(
        $Path,
        [IO.UnixFileMode]::UserRead -bor [IO.UnixFileMode]::UserWrite -bor
            [IO.UnixFileMode]::UserExecute
    )
}

function Get-PreparedBuildKey([string]$BuildRoot) {
    $material = [Collections.Generic.List[string]]::new()
    $material.Add("matrix=$(Get-FileSha256 $candidateMatrix)")
    foreach ($sourcePath in @(
        'tools/x4-verification/build-candidate-extension.ps1',
        'tools/x4-verification/contracts/candidate-build-manifest.v1.json',
        'tools/x4-verification/templates/candidate-content.xml',
        'tools/x4-verification/templates/candidate-entry.lua',
        'tools/x4-verification/templates/candidate-ui.xml',
        'tests/x4-candidates/lua/live_galaxy_candidate_runner.lua'
    )) {
        $material.Add("source/$sourcePath=$(Get-FileSha256 (Join-Path $root $sourcePath))")
    }
    $groups = @(Get-ChildItem -LiteralPath $BuildRoot -Directory | Sort-Object Name)
    if ($groups.Count -lt 1 -or $groups.Count -gt 16) { throw 'PREPARED_BUILD_GROUPS_INVALID' }
    foreach ($group in $groups) {
        if (($group.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw 'PREPARED_BUILD_REPARSE_REJECTED'
        }
        $manifestPath = Join-Path $group.FullName 'manifest/build-manifest.v1.json'
        if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
            throw 'PREPARED_BUILD_MANIFEST_MISSING'
        }
        $manifestItem = Get-Item -LiteralPath $manifestPath -Force
        if ($manifestItem.Length -gt 262144 -or
            ($manifestItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw 'PREPARED_BUILD_MANIFEST_INVALID'
        }
        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json -Depth 64
        $material.Add("manifest/$($group.Name)=$(Get-FileSha256 $manifestPath)")
        $generatedFiles = @($manifest.generated_files | Sort-Object path)
        if ($generatedFiles.Count -lt 1 -or $generatedFiles.Count -gt 16) {
            throw 'PREPARED_BUILD_FILES_INVALID'
        }
        [long]$totalBytes = 0
        foreach ($generated in $generatedFiles) {
            $logicalPath = [string]$generated.path
            if ($logicalPath -notmatch '^[a-zA-Z0-9._/-]+$' -or
                @($logicalPath -split '[\\/]+') -contains '..') {
                throw 'PREPARED_BUILD_PATH_INVALID'
            }
            $generatedPath = [IO.Path]::GetFullPath((Join-Path $group.FullName $logicalPath))
            if (-not $generatedPath.StartsWith(
                $group.FullName.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar,
                [StringComparison]::OrdinalIgnoreCase
            )) { throw 'PREPARED_BUILD_PATH_INVALID' }
            if (-not (Test-Path -LiteralPath $generatedPath -PathType Leaf)) {
                throw 'PREPARED_BUILD_FILE_MISSING'
            }
            $generatedItem = Get-Item -LiteralPath $generatedPath -Force
            if ($generatedItem.Length -gt 65536 -or
                ($generatedItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw 'PREPARED_BUILD_FILE_INVALID'
            }
            $totalBytes += $generatedItem.Length
            $material.Add("$($group.Name)/$($generated.path)=$(Get-FileSha256 $generatedPath)")
        }
        if ($totalBytes -gt 524288) { throw 'PREPARED_BUILD_FILES_INVALID' }
    }
    $bytes = [Text.Encoding]::UTF8.GetBytes(($material -join "`n"))
    return [Convert]::ToHexString(
        [Security.Cryptography.SHA256]::HashData($bytes)
    ).ToLowerInvariant()
}

function New-PreparedBuild([string]$Gate) {
    $stopwatch = [Diagnostics.Stopwatch]::StartNew()
    $status = 'fail'
    try {
        $script:preparedInvocationRoot = Join-Path ([IO.Path]::GetTempPath()) `
            ('live-galaxy-prepared-' + [guid]::NewGuid().ToString('N'))
        $null = New-Item -ItemType Directory -Path $script:preparedInvocationRoot
        Set-OwnerOnlyDirectory $script:preparedInvocationRoot
        $buildingRoot = Join-Path $script:preparedInvocationRoot 'building'
        $builderOutput = @(& pwsh -NoProfile -File $candidateBuilder `
            -BuildRoot $buildingRoot -MatrixPath $candidateMatrix)
        $builderExit = $LASTEXITCODE
        $builderOutput | Write-Output
        if ($builderExit -ne 0) { throw 'PREPARED_BUILD_GENERATION_FAILED' }
        $script:preparedBuildKey = Get-PreparedBuildKey $buildingRoot
        $script:preparedBuildRoot = Join-Path $script:preparedInvocationRoot $script:preparedBuildKey
        Move-Item -LiteralPath $buildingRoot -Destination $script:preparedBuildRoot
        Set-OwnerOnlyDirectory $script:preparedBuildRoot
        $status = 'pass'
    }
    finally {
        $stopwatch.Stop()
        [ordered]@{
            schema = 'x4-verification-stage-timing.v1'
            gate = $Gate
            stage_id = 'prepared-build'
            elapsed_ms = [long]$stopwatch.ElapsedMilliseconds
            status = $status
        } | ConvertTo-Json -Compress | Write-Output
    }
}

function Remove-PreparedBuild {
    if ($null -eq $script:preparedInvocationRoot) { return }
    try {
        Remove-Item -LiteralPath $script:preparedInvocationRoot -Recurse -Force
    }
    catch { throw 'PREPARED_BUILD_CLEANUP_FAILED' }
    if (Test-Path -LiteralPath $script:preparedInvocationRoot) {
        throw 'PREPARED_BUILD_CLEANUP_FAILED'
    }
    $script:preparedInvocationRoot = $null
    $script:preparedBuildRoot = $null
    $script:preparedBuildKey = $null
}

trap {
    $failure = $_
    Remove-PreparedBuild
    throw $failure
}

function Invoke-TimedStage {
    param(
        [Parameter(Mandatory)]
        [string]$Gate,
        [Parameter(Mandatory)]
        [string]$StageId,
        [Parameter(Mandatory)]
        [string]$Executable,
        [Parameter(Mandatory)]
        [string[]]$Arguments,
        [Parameter(Mandatory)]
        [string]$FailureMessage,
        [string[]]$ExpectedMarkers = @(),
        [string[]]$MissingMarkerMessages = @(),
        [string]$CaptureVariableName,
        [bool]$RequireOutput = $false,
        [string]$EmptyOutputMessage,
        [bool]$EmitTiming = $true
    )

    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $status = 'fail'
    try {
        $stageOutput = @(& $Executable @Arguments)
        $exitCode = $LASTEXITCODE
        $stageOutput | Write-Output
        if ($exitCode -ne 0) {
            throw $FailureMessage
        }
        if ($RequireOutput -and $stageOutput.Count -eq 0) {
            throw $EmptyOutputMessage
        }
        for ($index = 0; $index -lt $ExpectedMarkers.Count; $index++) {
            if ($stageOutput -notcontains $ExpectedMarkers[$index]) {
                $markerMessage = if ($index -lt $MissingMarkerMessages.Count) {
                    $MissingMarkerMessages[$index]
                }
                else {
                    $FailureMessage
                }
                throw $markerMessage
            }
        }
        if (-not [string]::IsNullOrWhiteSpace($CaptureVariableName)) {
            Set-Variable -Scope 1 -Name $CaptureVariableName -Value $stageOutput
        }
        $status = 'pass'
    }
    finally {
        $stopwatch.Stop()
        if ($EmitTiming) {
            [ordered]@{
                schema = 'x4-verification-stage-timing.v1'
                gate = $Gate
                stage_id = $StageId
                elapsed_ms = [long]$stopwatch.ElapsedMilliseconds
                status = $status
            } | ConvertTo-Json -Compress | Write-Output
        }
    }
}

if ($Suite -eq 'x4-verification-reuse-contract') {
    New-PreparedBuild $Suite
    $preparedArguments = @(
        '-PreparedBuildRoot', $script:preparedBuildRoot,
        '-PreparedBuildKey', $script:preparedBuildKey
    )
    Invoke-TimedStage -Gate $Suite -StageId 'reuse-retention' `
        -Executable 'pwsh' -Arguments (@('-NoProfile', '-File', $evidenceRetentionContract, '-Case', 'reuse-contract') + $preparedArguments) `
        -FailureMessage 'X4 prepared-build retention reuse contract failed.'
    Invoke-TimedStage -Gate $Suite -StageId 'reuse-runtime' `
        -Executable 'pwsh' -Arguments (@('-NoProfile', '-File', $candidatePackageRuntimeContract, '-Case', 'reuse-contract') + $preparedArguments) `
        -FailureMessage 'X4 prepared-build runtime reuse contract failed.'
    Invoke-TimedStage -Gate $Suite -StageId 'reuse-adversarial' `
        -Executable 'pwsh' -Arguments (@('-NoProfile', '-File', $candidatePackageAdversarial, '-Case', 'reuse-contract') + $preparedArguments) `
        -FailureMessage 'X4 prepared-build adversarial reuse contract failed.'
    Remove-PreparedBuild
    Write-Output 'PASS: x4-verification-reuse-contract'
    exit 0
}

if ($Suite -eq 'x4-verification-fast') {
    New-PreparedBuild $Suite
    $preparedArguments = @(
        '-PreparedBuildRoot', $script:preparedBuildRoot,
        '-PreparedBuildKey', $script:preparedBuildKey
    )
    Invoke-TimedStage -Gate $Suite -StageId 'package-conformance-packaged-path' `
        -Executable 'pwsh' -Arguments @('-NoProfile', '-File', $packageConformanceContract, '-Case', 'packaged-path', '-SkipAggregateRegistration') `
        -FailureMessage 'X4 package conformance contract failed.' `
        -ExpectedMarkers @('package conformance contract passed: packaged-path') `
        -MissingMarkerMessages @('X4 package conformance PASS marker is missing: packaged-path')
    Invoke-TimedStage -Gate $Suite -StageId 'candidate-isolation-all' `
        -Executable 'pwsh' -Arguments @('-NoProfile', '-File', $candidateIsolationContract, '-Case', 'all') `
        -FailureMessage 'X4 candidate isolation contract failed.'
    Invoke-TimedStage -Gate $Suite -StageId 'candidate-runtime-adapters' `
        -Executable 'pwsh' -Arguments (@('-NoProfile', '-File', $candidatePackageRuntimeContract, '-Case', 'adapters') + $preparedArguments) `
        -FailureMessage 'X4 candidate-package runtime contract failed.' `
        -ExpectedMarkers @('candidate-package-runtime: adapters PASS')
    Invoke-TimedStage -Gate $Suite -StageId 'evidence-retention-retention' `
        -Executable 'pwsh' -Arguments (@('-NoProfile', '-File', $evidenceRetentionContract, '-Case', 'retention') + $preparedArguments) `
        -FailureMessage 'X4 evidence-retention contract failed: retention'
    Invoke-TimedStage -Gate $Suite -StageId 'evidence-retention-preallocation-bounds' `
        -Executable 'pwsh' -Arguments @('-NoProfile', '-File', $evidenceRetentionContract, '-Case', 'preallocation-bounds') `
        -FailureMessage 'X4 evidence-retention contract failed: preallocation-bounds' `
        -ExpectedMarkers @('PASS: shared open-handle race contract') `
        -MissingMarkerMessages @('X4 open-handle race marker was not reached.')
    Remove-PreparedBuild
    Write-Output 'PASS: x4-verification-fast'
    exit 0
}

if ($Suite -eq 'x4-verification') {
    New-PreparedBuild $Suite
    $preparedArguments = @(
        '-PreparedBuildRoot', $script:preparedBuildRoot,
        '-PreparedBuildKey', $script:preparedBuildKey
    )
}

if ($Suite -in @('all', 'x4-admission', 'x4-verification')) {
    foreach ($admissionCase in @('dossier', 'negative-fixtures', 'admission', 'evidence-chain')) {
        $expectedMarkers = if ($admissionCase -eq 'admission') {
            @('PASS: owner override admission contract')
        }
        else { @() }
        $missingMarkerMessages = if ($admissionCase -eq 'admission') {
            @('X4 owner override admission marker was not reached.')
        }
        else { @() }
        Invoke-TimedStage -Gate $Suite -StageId "admission-$admissionCase" `
            -Executable 'pwsh' -Arguments @('-NoProfile', '-File', $admissionContract, '-Case', $admissionCase) `
            -FailureMessage "X4 admission contract failed: $admissionCase" `
            -ExpectedMarkers $expectedMarkers -MissingMarkerMessages $missingMarkerMessages `
            -EmitTiming ($Suite -eq 'x4-verification')
    }
    if ($Suite -eq 'x4-admission') {
        exit 0
    }
}

if ($Suite -in @('all', 'x4-verification')) {
    Invoke-TimedStage -Gate $Suite -StageId 'owner-authority-root-delegation' `
        -Executable 'pwsh' -Arguments @('-NoProfile', '-File', $ownerAuthorityContract, '-Case', 'root-delegation') `
        -FailureMessage 'X4 owner authority root-delegation contract failed.' `
        -EmitTiming ($Suite -eq 'x4-verification')
    Invoke-TimedStage -Gate $Suite -StageId 'owner-authority-adversarial' `
        -Executable 'pwsh' -Arguments @('-NoProfile', '-File', $ownerAuthorityAdversarial) `
        -FailureMessage 'X4 owner authority adversarial contract failed.' `
        -EmitTiming ($Suite -eq 'x4-verification')
    Invoke-TimedStage -Gate $Suite -StageId 'candidate-build-all' `
        -Executable 'pwsh' -Arguments @('-NoProfile', '-File', $candidateBuildContract, '-Case', 'all') `
        -FailureMessage 'X4 candidate-build aggregate contract failed.' `
        -EmitTiming ($Suite -eq 'x4-verification')
    foreach ($retentionCase in @('retention', 'retention-platform', 'handback', 'retention-admission', 'preallocation-bounds')) {
        $expectedMarkers = if ($retentionCase -eq 'preallocation-bounds') {
            @('PASS: shared open-handle race contract')
        }
        else { @() }
        $missingMarkerMessages = if ($retentionCase -eq 'preallocation-bounds') {
            @('X4 open-handle race marker was not reached.')
        }
        else { @() }
        Invoke-TimedStage -Gate $Suite -StageId "evidence-retention-$retentionCase" `
            -Executable 'pwsh' -Arguments (@('-NoProfile', '-File', $evidenceRetentionContract, '-Case', $retentionCase) + `
                $(if ($Suite -eq 'x4-verification' -and $retentionCase -ne 'preallocation-bounds') { $preparedArguments } else { @() })) `
            -FailureMessage "X4 evidence-retention contract failed: $retentionCase" `
            -ExpectedMarkers $expectedMarkers -MissingMarkerMessages $missingMarkerMessages `
            -EmitTiming ($Suite -eq 'x4-verification')
    }
    Invoke-TimedStage -Gate $Suite -StageId 'evidence-chain-adversarial' `
        -Executable 'pwsh' -Arguments @('-NoProfile', '-File', $evidenceChainAdversarial) `
        -FailureMessage 'X4 held-out evidence-chain adversarial contract failed.' `
        -EmitTiming ($Suite -eq 'x4-verification')
}

if ($Suite -in @('all', 'x4-candidate-runner', 'x4-verification')) {
    Invoke-TimedStage -Gate $Suite -StageId 'candidate-isolation-all' `
        -Executable 'pwsh' -Arguments @('-NoProfile', '-File', $candidateIsolationContract, '-Case', 'all') `
        -FailureMessage 'X4 candidate isolation contract failed.' `
        -EmitTiming ($Suite -eq 'x4-verification')
}

if ($Suite -in @('all', 'x4-verification')) {
    Invoke-TimedStage -Gate $Suite -StageId 'candidate-runtime-all' `
        -Executable 'pwsh' -Arguments (@('-NoProfile', '-File', $candidatePackageRuntimeContract, '-Case', 'all') + `
            $(if ($Suite -eq 'x4-verification') { $preparedArguments } else { @() })) `
        -FailureMessage 'X4 candidate-package runtime contract failed.' `
        -ExpectedMarkers @('candidate-package-runtime: adapters PASS') `
        -EmitTiming ($Suite -eq 'x4-verification')
    Invoke-TimedStage -Gate $Suite -StageId 'candidate-adversarial-all' `
        -Executable 'pwsh' -Arguments (@('-NoProfile', '-File', $candidatePackageAdversarial, '-Case', 'all') + `
            $(if ($Suite -eq 'x4-verification') { $preparedArguments } else { @() })) `
        -FailureMessage 'X4 candidate-package adversarial contract failed.' `
        -ExpectedMarkers @('candidate-package-adversarial: PASS') `
        -EmitTiming ($Suite -eq 'x4-verification')
}

if ($Suite -in @('all', 'x4-package-conformance', 'x4-verification')) {
    $packageCase = if ($Suite -eq 'x4-package-conformance') { 'packaged-path' } else { 'all' }
    $packageArguments = @('-NoProfile', '-File', $packageConformanceContract, '-Case', $packageCase)
    if ($packageCase -eq 'all') { $packageArguments += '-SkipAggregateRegistration' }
    Invoke-TimedStage -Gate $Suite -StageId "package-conformance-$packageCase" `
        -Executable 'pwsh' -Arguments $packageArguments `
        -FailureMessage 'X4 package conformance contract failed.' `
        -ExpectedMarkers @("package conformance contract passed: $packageCase") `
        -MissingMarkerMessages @("X4 package conformance PASS marker is missing: $packageCase") `
        -EmitTiming ($Suite -eq 'x4-verification')
    if ($Suite -eq 'x4-package-conformance') {
        exit 0
    }
}

if ($Suite -eq 'component_discovery_guard') {
    & powershell -NoProfile -File $componentDiscoveryGuard -SelfTest
    if ($LASTEXITCODE -ne 0) { throw 'Component discovery package guard failed.' }
    foreach ($productionPath in $componentDiscoveryProductionPaths) {
        & powershell -NoProfile -File $componentDiscoveryGuard -ProductionPath $productionPath
        if ($LASTEXITCODE -ne 0) { throw "Component discovery package guard failed: $productionPath" }
    }
    exit 0
}

if ($Suite -eq 'component_discovery') {
    & powershell -NoProfile -File (Join-Path $PSScriptRoot 'component_discovery_binding_contract.ps1')
    if ($LASTEXITCODE -ne 0) { throw 'Component discovery binding contract failed.' }
    & powershell -NoProfile -File $componentDiscoveryGuard -SelfTest
    if ($LASTEXITCODE -ne 0) { throw 'Component discovery package guard self-test failed.' }
    foreach ($productionPath in $componentDiscoveryProductionPaths) {
        & powershell -NoProfile -File $componentDiscoveryGuard -ProductionPath $productionPath
        if ($LASTEXITCODE -ne 0) { throw "Component discovery package guard failed: $productionPath" }
    }
}

$lockPath = Join-Path $root 'tools/lua-runner.lock.json'
if (-not (Test-Path -LiteralPath $lockPath -PathType Leaf)) {
    throw "Lua runner lock is missing: $lockPath"
}
$lock = Get-Content -LiteralPath $lockPath -Raw | ConvertFrom-Json
if ([string]::IsNullOrWhiteSpace($lock.executableVersion) -or
    [string]::IsNullOrWhiteSpace($lock.materialization.executableRelativePath)) {
    throw 'Lua runner lock has required empty fields.'
}

function Test-LockedLua([string]$Candidate, [string]$Source) {
    if ([string]::IsNullOrWhiteSpace($Candidate) -or
        -not (Test-Path -LiteralPath $Candidate -PathType Leaf)) {
        return $null
    }

    $previousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        $reported = (& $Candidate -v 2>&1 | ForEach-Object {
            if ($_ -is [System.Management.Automation.ErrorRecord]) {
                $_.Exception.Message
            }
            else {
                $_.ToString()
            }
        } | Out-String).Trim()
    }
    finally {
        $ErrorActionPreference = $previousErrorActionPreference
    }
    if ($LASTEXITCODE -ne 0 -or
        $reported -notmatch "(?m)^$([regex]::Escape($lock.executableVersion))(?:\s|$)") {
        throw "Lua runner from $Source does not match lock '$($lock.executableVersion)': $reported"
    }
    return (Resolve-Path -LiteralPath $Candidate).Path
}

$lua = Test-LockedLua $LuaPath '-LuaPath'
if ($null -eq $lua) {
    $lua = Test-LockedLua $env:LIVE_GALAXY_LUA 'LIVE_GALAXY_LUA'
}
if ($null -eq $lua) {
    $lua = Test-LockedLua (Join-Path $root $lock.materialization.executableRelativePath) 'project-local lock path'
}
if ($null -eq $lua) {
    $pathLua = Get-Command lua -ErrorAction SilentlyContinue
    if ($null -ne $pathLua) {
        $lua = Test-LockedLua $pathLua.Source 'PATH'
    }
}
if ($null -eq $lua) {
    throw "Lua runner unavailable. Set -LuaPath or LIVE_GALAXY_LUA, or run tools/provision-lua.ps1; every candidate must match '$($lock.executableVersion)'."
}

$tests = if ($Suite -eq 'x4_discovery') {
    @('x4_discovery_contract.lua')
} elseif ($Suite -eq 'component_discovery') {
    @('component_discovery_contract.lua')
} elseif ($Suite -in @('x4-candidate-runner', 'x4-verification')) {
    @('x4_candidate_runner_contract.lua', 'x4_candidate_runner_adversarial.lua')
} else {
    @(
        Get-ChildItem $PSScriptRoot -Filter '*_contract.lua' | ForEach-Object Name
        'x4_candidate_runner_adversarial.lua'
    )
}

$candidateMarkers = @()
foreach ($test in $tests) {
    $path = Join-Path $PSScriptRoot $test
    $modulePath = Join-Path $root 'extensions\?.lua'
    $moduleInitPath = Join-Path $root 'extensions\?\init.lua'
    $extensionLuaPath = Join-Path $root 'extensions\live_galaxy\lua\?.lua'
    $luaCommand = "package.path = [[${modulePath};${moduleInitPath};${extensionLuaPath};]] .. package.path local cases = dofile([[${path}]]) assert(type(cases) == 'table', 'MALFORMED_LUA_CASE_TABLE') local count = 0 for name, case in pairs(cases) do assert(type(name) == 'string' and type(case) == 'function', 'MALFORMED_LUA_CASE_TABLE') count = count + 1 end assert(count > 0, 'EMPTY_LUA_CASE_TABLE') for name, case in pairs(cases) do case() print('PASS x4-candidate-runner:' .. name) end print('CASES ${test}: ' .. count)"
    Invoke-TimedStage -Gate $Suite -StageId "lua-$([IO.Path]::GetFileNameWithoutExtension($test))" `
        -Executable $lua -Arguments @('-e', $luaCommand) `
        -FailureMessage "Lua contract failed: $test" `
        -CaptureVariableName 'caseOutput' -RequireOutput $true `
        -EmptyOutputMessage "Lua contract produced no behavior markers: $test" `
        -EmitTiming ($Suite -eq 'x4-verification')
    if ($test -in @('x4_candidate_runner_contract.lua', 'x4_candidate_runner_adversarial.lua')) {
        $candidateMarkers += $caseOutput
    }
}

if ($Suite -in @('all', 'x4-candidate-runner', 'x4-verification')) {
    $requiredCandidateMarkers = @(
        'fixed_sha256_vectors_reject_same_length_tampering',
        'emits_one_candidate_as_three_digest_bound_jsonl_stages',
        'isolates_exceptions_malformed_results_and_work_unit_exhaustion',
        'lua_instruction_watchdog_preempts_cooperative_execution_and_continues',
        'never_passes_a_valid_but_unexpected_effect',
        'records_protected_contract_and_effect_failure_reasons_then_continues',
        'rejects_missing_identity_bounds_and_digest_failures_with_exact_codes',
        'independent_contract_rejects_collapsed_verdicts_and_noncanonical_order',
        'rejects_candidate_owned_authority_and_unattested_native_dispatch',
        'candidate_authority_fields_never_execute',
        'candidate_metatable_authority_never_executes',
        'direct_native_and_dynamic_authority_fail_closed',
        'runner_source_has_no_direct_native_binding'
    )
    foreach ($marker in $requiredCandidateMarkers) {
        if ($candidateMarkers -notcontains "PASS x4-candidate-runner:$marker") {
            throw "Required candidate behavior marker is missing: $marker"
        }
    }
}

if ($Suite -eq 'x4-verification') {
    Remove-PreparedBuild
    Write-Output 'PASS: x4-verification'
}
