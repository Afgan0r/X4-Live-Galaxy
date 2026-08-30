[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string]$BuildRoot,
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string]$MatrixPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:reasonCode = 'INTERNAL_BUILD_ERROR'
$script:createdGroups = @()
$repositoryRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$manifestContractPath = Join-Path $repositoryRoot 'tools/x4-verification/contracts/candidate-build-manifest.v1.json'
$packageContractPath = Join-Path $repositoryRoot 'tools/x4-verification/contracts/package-conformance.v1.json'
$packageConformanceScriptPath = Join-Path $repositoryRoot 'tools/x4-verification/x4-package-conformance.ps1'
$dossierPath = Join-Path $repositoryRoot 'tools/x4-verification/contracts/phase-05.1-dossier.v1.json'
$registryPath = Join-Path $repositoryRoot 'tools/x4-verification/contracts/known-failures.v1.json'
$coveragePath = Join-Path $repositoryRoot 'tools/x4-verification/contracts/coverage.v1.json'
$runtimeEvidencePath = Join-Path $repositoryRoot 'tools/x4-verification/contracts/runtime-evidence.v1.json'
$runnerSourcePath = Join-Path $repositoryRoot 'tests/x4-candidates/lua/live_galaxy_candidate_runner.lua'
$entrypointTemplatePath = Join-Path $repositoryRoot 'tools/x4-verification/templates/candidate-entry.lua'
$dispatcherSourcePath = Join-Path $repositoryRoot 'tools/x4-verification/run-candidate-package.ps1'
$adapterSourcePath = Join-Path $repositoryRoot 'tools/x4-verification/candidate-adapters.psm1'
$attestationModulePath = Join-Path $repositoryRoot 'tools/x4-verification/producer-attestation.psm1'
$workerSourcePath = Join-Path $repositoryRoot 'tools/x4-verification/isolation/candidate-worker.ps1'
$launcherSourcePath = Join-Path $repositoryRoot 'tools/x4-verification/isolation/invoke-candidate-worker.ps1'
$boundedReaderSourcePath = Join-Path $repositoryRoot 'tools/x4-verification/bounded-file.psm1'
$workerProtocolPath = Join-Path $repositoryRoot 'tools/x4-verification/contracts/candidate-worker-protocol.v1.json'
$ownerRootAnchorPath = Join-Path $repositoryRoot 'tools/x4-verification/contracts/owner-root-anchor.v1.json'
$contentTemplatePath = Join-Path $repositoryRoot 'tools/x4-verification/templates/candidate-content.xml'
$uiTemplatePath = Join-Path $repositoryRoot 'tools/x4-verification/templates/candidate-ui.xml'
$publicPackageRoot = Join-Path $repositoryRoot 'extensions/live_galaxy'
$profileFields = @('content_profile', 'ui_registration_profile', 'entrypoint', 'import_root', 'binding_profile')
$expectedCandidateIds = @(
    'p051-cadence-seta',
    'p051-lifecycle-reload',
    'p051-mod-stack-compatibility',
    'p051-native-count-fill-runtime',
    'p051-native-fill-completeness',
    'p051-native-identity-closure',
    'p051-native-volume-envelope'
)
$expectedSourceExclusions = @(
    'equate-frame-budget-with-native-allocation-bound',
    'invent-native-offset-pagination',
    'select-production-ceiling-from-one-run',
    'treat-emission-paging-as-discovery-paging',
    'truncate-owner-scope'
)
$requiredCandidateFields = @(
    'id', 'status', 'source_action_only', 'question', 'expected_result',
    'evidence_ids', 'failure_classifications', 'bounded_steps',
    'stop_conditions', 'verdict_axes', 'build_group', 'build_profile',
    'build_profile_digest', 'exclusive_build', 'conflicts_with'
)

Import-Module $attestationModulePath -Force
Import-Module $boundedReaderSourcePath -Force

function Fail([string]$Code) {
    $script:reasonCode = $Code
    throw [InvalidOperationException]::new($Code)
}

function Get-Sha256([byte[]]$Bytes) {
    return [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant()
}

function Read-BoundedBytes([string]$Path, [int]$Maximum, [string]$FailureCode) {
    try {
        return (Read-BoundedFile $Path $Maximum 'MISSING_INPUT' $FailureCode `
            'PATH_IDENTITY_CHANGED' 'INPUT_REPARSE_POINT_REJECTED').Bytes
    }
    catch { Fail ([string]$_.Exception.Message) }
}

function Get-FileDigest([string]$Path, [int]$Maximum = 262144, [string]$FailureCode = 'INPUT_BYTES_EXCEEDED') {
    return Get-Sha256 (Read-BoundedBytes $Path $Maximum $FailureCode)
}

function Set-OwnerOnly([string]$Path, [bool]$Directory) {
    if (-not $IsWindows) {
        [IO.File]::SetUnixFileMode(
            $Path,
            $(if ($Directory) {
                [IO.UnixFileMode]::UserRead -bor [IO.UnixFileMode]::UserWrite -bor [IO.UnixFileMode]::UserExecute
            } else { [IO.UnixFileMode]::UserRead -bor [IO.UnixFileMode]::UserWrite })
        )
        return
    }
    $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User
    $grant = if ($Directory) { "*$($sid.Value):(OI)(CI)F" } else { "*$($sid.Value):F" }
    $null = & icacls.exe $Path /inheritance:r /grant:r $grant
    if ($LASTEXITCODE -ne 0) { Fail 'OWNER_ONLY_PERMISSION_FAILED' }
}

function Read-Json([string]$Path, [string]$Schema) {
    $bytes = Read-BoundedBytes $Path 262144 'INPUT_BYTES_EXCEEDED'
    try { $value = [Text.Encoding]::UTF8.GetString($bytes) | ConvertFrom-Json -Depth 64 }
    catch { Fail 'INVALID_JSON' }
    if ($value.schema_version -ne $Schema) { Fail 'UNSUPPORTED_SCHEMA' }
    return $value
}

function Require-Text($Value) {
    if ($Value -isnot [string] -or [string]::IsNullOrWhiteSpace($Value) -or $Value.Length -gt 1024) {
        Fail 'CANDIDATE_TEXT_INVALID'
    }
}

function Get-ProfileDigest($Profile) {
    $lines = foreach ($field in $profileFields) {
        if ($null -eq $Profile.PSObject.Properties[$field]) { Fail 'PROFILE_FIELD_MISSING' }
        $value = [string]$Profile.$field
        if ([string]::IsNullOrWhiteSpace($value) -or $value.Length -gt 256) { Fail 'PROFILE_FIELD_INVALID' }
        "$field=$value"
    }
    if (@($Profile.PSObject.Properties).Count -ne $profileFields.Count) { Fail 'PROFILE_FIELD_UNDECLARED' }
    return Get-Sha256 ([Text.Encoding]::UTF8.GetBytes(($lines -join "`n")))
}

function Test-ContainedPath([string]$Candidate, [string]$Container) {
    $candidateFull = [IO.Path]::GetFullPath($Candidate).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    $containerFull = [IO.Path]::GetFullPath($Container).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    return $candidateFull.Equals($containerFull, [StringComparison]::OrdinalIgnoreCase) -or
        $candidateFull.StartsWith($containerFull + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)
}

function Resolve-NoReparseDestination([string]$Path) {
    $full = [IO.Path]::GetFullPath($Path).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    $root = [IO.Path]::GetPathRoot($full)
    $relative = $full.Substring($root.Length).TrimStart([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    $segments = if ([string]::IsNullOrWhiteSpace($relative)) { @() } else { @($relative -split '[\\/]+' ) }
    $current = $root
    $existingCount = 0
    foreach ($segment in $segments) {
        $next = Join-Path $current $segment
        if (-not (Test-Path -LiteralPath $next)) { break }
        $item = Get-Item -LiteralPath $next -Force
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { Fail 'DESTINATION_REPARSE_POINT_REJECTED' }
        if (-not $item.PSIsContainer) { Fail 'DESTINATION_NOT_DIRECTORY' }
        $current = $item.FullName
        $existingCount += 1
    }
    $resolved = if (Test-Path -LiteralPath $current) { (Resolve-Path -LiteralPath $current).Path } else { $current }
    for ($index = $existingCount; $index -lt $segments.Count; $index += 1) {
        $resolved = Join-Path $resolved $segments[$index]
    }
    return [IO.Path]::GetFullPath($resolved).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
}

function Resolve-SafeBuildRoot([string]$Path) {
    $full = Resolve-NoReparseDestination $Path
    $volumeRoot = [IO.Path]::GetPathRoot($full).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    if ([string]::IsNullOrWhiteSpace($full) -or $full.Equals($volumeRoot, [StringComparison]::OrdinalIgnoreCase)) { Fail 'FILESYSTEM_ROOT_REJECTED' }
    if (Test-ContainedPath $full $repositoryRoot) { Fail 'REPOSITORY_DESTINATION_REJECTED' }
    if (Test-ContainedPath $full $publicPackageRoot) { Fail 'PUBLIC_PACKAGE_DESTINATION_REJECTED' }
    if ($full -match '(?i)[\\/]steamapps[\\/]common[\\/]X4 Foundations(?:[\\/]|$)') { Fail 'GAME_INSTALLATION_DESTINATION_REJECTED' }
    if ($full -match '(?i)[\\/]X4 Foundations[\\/]extensions(?:[\\/]|$)' -or
        $full -match '(?i)[\\/]extensions[\\/]live_galaxy(?:[\\/]|$)') {
        Fail 'PUBLIC_RUNTIME_DESTINATION_REJECTED'
    }
    if ($full -match '(?i)[\\/]Egosoft[\\/]X4[\\/][0-9]+[\\/]save(?:[\\/]|$)') {
        Fail 'GAME_SAVE_DESTINATION_REJECTED'
    }
    if ($full.Length -gt 240) { Fail 'DESTINATION_PATH_EXCEEDED' }
    return $full
}

function Test-StringArray($Values, [int]$Minimum, [int]$Maximum, [string]$Code) {
    $array = @($Values)
    if ($array.Count -lt $Minimum -or $array.Count -gt $Maximum) { Fail $Code }
    foreach ($value in $array) {
        if ($value -isnot [string] -or [string]::IsNullOrWhiteSpace($value) -or $value.Length -gt 256) { Fail $Code }
    }
    if (@($array | Sort-Object -Unique).Count -ne $array.Count) { Fail $Code }
}

function Assert-ValidMatrix($Matrix) {
    if ($Matrix.status -ne 'runtime-pending' -or $Matrix.evidence_classification -ne 'scaffold-only') { Fail 'MATRIX_STATUS_INVALID' }
    if ($Matrix.bounds.max_candidates -ne 7 -or @($Matrix.candidates).Count -ne 7) { Fail 'CANDIDATE_SET_INVALID' }
    if ((@($Matrix.candidates.id) -join '|') -ne ($expectedCandidateIds -join '|')) { Fail 'CANDIDATE_SET_INVALID' }
    if (@($Matrix.build_groups).Count -lt 1 -or @($Matrix.build_groups).Count -gt $Matrix.bounds.max_groups) { Fail 'GROUP_COUNT_INVALID' }
    if ((@($Matrix.build_groups.id) -join '|') -ne ((@($Matrix.build_groups.id) | Sort-Object) -join '|')) { Fail 'GROUP_ORDER_INVALID' }
    Test-StringArray $Matrix.source_resolvable_exclusions 5 5 'SOURCE_EXCLUSIONS_INVALID'
    if ((@($Matrix.source_resolvable_exclusions) -join '|') -ne ($expectedSourceExclusions -join '|')) { Fail 'SOURCE_EXCLUSIONS_INVALID' }

    $inputs = @{
        dossier = $dossierPath
        registry = $registryPath
        coverage = $coveragePath
        runtime_evidence = $runtimeEvidencePath
        package_conformance = $packageContractPath
    }
    foreach ($inputId in $inputs.Keys) {
        $row = @($Matrix.source_inputs | Where-Object { $_.id -eq $inputId })
        if ($row.Count -ne 1 -or $row[0].sha256 -ne (Get-FileDigest $inputs[$inputId])) { Fail 'SOURCE_DIGEST_MISMATCH' }
    }

    $byId = @{}
    foreach ($candidate in @($Matrix.candidates)) {
        foreach ($field in $requiredCandidateFields) {
            if ($candidate.PSObject.Properties.Name -notcontains $field) { Fail 'CANDIDATE_FIELD_MISSING' }
        }
        if ($byId.ContainsKey($candidate.id)) { Fail 'DUPLICATE_CANDIDATE' }
        $byId[$candidate.id] = $candidate
        if ($candidate.status -ne 'runtime-pending' -or $candidate.source_action_only -ne $false) { Fail 'CANDIDATE_STATUS_INVALID' }
        Require-Text $candidate.question
        Require-Text $candidate.expected_result
        Test-StringArray $candidate.evidence_ids 1 16 'CANDIDATE_EVIDENCE_INVALID'
        Test-StringArray $candidate.failure_classifications 1 8 'CANDIDATE_FAILURE_CLASSES_INVALID'
        Test-StringArray $candidate.bounded_steps 1 $Matrix.bounds.max_steps_per_candidate 'CANDIDATE_STEPS_INVALID'
        Test-StringArray $candidate.stop_conditions 1 $Matrix.bounds.max_stop_conditions_per_candidate 'CANDIDATE_STOPS_INVALID'
        if ((@($candidate.verdict_axes) -join '|') -ne 'execution|contract|effect') { Fail 'CANDIDATE_VERDICT_AXES_INVALID' }
        if ($candidate.exclusive_build -isnot [bool]) { Fail 'CANDIDATE_EXCLUSIVITY_INVALID' }
        if ($candidate.build_profile_digest -ne (Get-ProfileDigest $candidate.build_profile)) { Fail 'PROFILE_DIGEST_MISMATCH' }
        if ($candidate.build_profile.entrypoint -notmatch '^lua/[a-z0-9_/-]+\.lua$' -or
            $candidate.build_profile.import_root -notmatch '^[a-z0-9_/-]+/$') { Fail 'PROFILE_PATH_INVALID' }
        Test-StringArray $candidate.conflicts_with 0 6 'CONFLICT_SET_INVALID'
        if (@($candidate.conflicts_with) -contains $candidate.id) { Fail 'CONFLICT_SET_INVALID' }
    }
    foreach ($candidate in @($Matrix.candidates)) {
        foreach ($conflictId in @($candidate.conflicts_with)) {
            if (-not $byId.ContainsKey($conflictId)) { Fail 'UNDECLARED_CONFLICT' }
            if (-not (@($byId[$conflictId].conflicts_with) -contains $candidate.id)) { Fail 'ASYMMETRIC_CONFLICT' }
        }
    }

    $groups = @{}
    $membershipCounts = @{}
    foreach ($candidateId in $expectedCandidateIds) { $membershipCounts[$candidateId] = 0 }
    foreach ($group in @($Matrix.build_groups)) {
        if ($group.id -isnot [string] -or $group.id -notmatch '^[a-z0-9][a-z0-9-]{0,63}$') { Fail 'GROUP_ID_INVALID' }
        if ($groups.ContainsKey($group.id)) { Fail 'DUPLICATE_GROUP' }
        $groups[$group.id] = $group
        $members = @($group.candidate_ids)
        if ($members.Count -eq 0 -or ($members -join '|') -ne (($members | Sort-Object) -join '|')) { Fail 'GROUP_MEMBERS_INVALID' }
        foreach ($memberId in $members) {
            if (-not $byId.ContainsKey($memberId)) { Fail 'UNDECLARED_GROUP_MEMBER' }
            $membershipCounts[$memberId] = [int]$membershipCounts[$memberId] + 1
            $member = $byId[$memberId]
            if ($member.build_group -ne $group.id -or $member.build_profile_digest -ne $group.build_profile_digest) { Fail 'GROUP_PROFILE_MISMATCH' }
        }
    }
    foreach ($candidate in @($Matrix.candidates)) {
        if (-not $groups.ContainsKey($candidate.build_group)) { Fail 'MISSING_GROUP' }
        $members = @($groups[$candidate.build_group].candidate_ids)
        if ($membershipCounts[$candidate.id] -ne 1 -or $members -notcontains $candidate.id) {
            Fail 'GROUP_MEMBERSHIP_INVALID'
        }
        if ($candidate.exclusive_build -eq $true) {
            if ($members.Count -ne 1 -or @($candidate.conflicts_with).Count -ne 6) { Fail 'EXCLUSIVE_GROUP_INVALID' }
        }
        foreach ($other in @($Matrix.candidates | Where-Object { $_.id -gt $candidate.id })) {
            $conflict = @($candidate.conflicts_with) -contains $other.id
            $shareable = $candidate.build_profile_digest -eq $other.build_profile_digest -and
                $candidate.exclusive_build -ne $true -and $other.exclusive_build -ne $true -and -not $conflict
            if ($shareable -and $candidate.build_group -ne $other.build_group) { Fail 'NONDETERMINISTIC_PARTITION' }
            if ($candidate.build_group -eq $other.build_group -and -not $shareable) { Fail 'CONFLICTING_SAME_GROUP' }
        }
    }
}

function Write-Utf8NoBom([string]$Path, [string]$Value) {
    [IO.File]::WriteAllText($Path, $Value, [Text.UTF8Encoding]::new($false))
}

function Write-Json([string]$Path, $Value) {
    Write-Utf8NoBom $Path ($Value | ConvertTo-Json -Depth 64)
}

function Write-CanonicalJson([string]$Path, $Value) {
    [IO.File]::WriteAllBytes(
        $Path,
        (producer-attestation\ConvertTo-CanonicalJsonBytes $Value)
    )
}

function New-GeneratedConformanceContract($Base, [string]$PackageId, $Group, $Candidate) {
    return [ordered]@{
        schema_version = $Base.schema_version
        contract_id = "candidate-$($Group.id)"
        package_id = $PackageId
        required_content_dependency = $Base.required_content_dependency
        required_ui_dependency = $Base.required_ui_dependency
        required_environment = $Base.required_environment
        required_entrypoint = $Candidate.build_profile.entrypoint
        internal_module_prefix = $Candidate.build_profile.import_root
        external_modules = @($Base.external_modules)
        test_only_prefixes = @($Base.test_only_prefixes)
        native_binding = [ordered]@{
            module = $Base.native_binding.module
            binding_expression = $Base.native_binding.binding_expression
            policy = 'forbidden'
        }
        bounds = $Base.bounds
        admission_dimensions = @($Base.admission_dimensions)
        admission_failure_classes = @($Base.admission_failure_classes)
    }
}

function New-Entrypoint([string]$GroupId, [string]$BuildId) {
    $safeGroup = $GroupId -replace '[^a-z0-9_]', '_'
    return (Get-Content -LiteralPath $entrypointTemplatePath -Raw).
        Replace('{{BUILD_ID}}', $BuildId).
        Replace('{{GROUP_ID}}', $GroupId).
        Replace('{{SAFE_GROUP_ID}}', $safeGroup)
}

function Invoke-Conformance([string]$GroupRoot) {
    $generatedContractPath = Join-Path $GroupRoot 'manifest/package-conformance.v1.json'
    $output = @(& pwsh -NoProfile -File $packageConformanceScriptPath `
        -PackageRoot $GroupRoot -ContractPath $generatedContractPath `
        -DossierPath $dossierPath -CoveragePath $coveragePath 2>&1)
    if ($LASTEXITCODE -ne 0 -or $output.Count -ne 1) { Fail 'PACKAGE_CONFORMANCE_FAILED' }
    try { $result = $output[0].ToString() | ConvertFrom-Json -Depth 32 -DateKind String }
    catch { Fail 'PACKAGE_CONFORMANCE_FAILED' }
    if ($result.schema_version -ne 'x4-package-conformance-result.v1' -or
        $result.verdict -ne 'conformant' -or $result.classification -ne 'local-only' -or
        $result.evidence_level -ne 'packaged-static' -or $null -ne $result.native_binding_path) {
        Fail 'PACKAGE_CONFORMANCE_FAILED'
    }
    $canonical = $result | ConvertTo-Json -Compress -Depth 32
    return [pscustomobject]@{ Value = $result; Digest = Get-Sha256 ([Text.Encoding]::UTF8.GetBytes($canonical)) }
}

function New-GroupBuild($Matrix, $Group, [string]$Destination, $ManifestContract, $PackageContract, [string]$MatrixDigest) {
    $members = @($Matrix.candidates | Where-Object { @($Group.candidate_ids) -contains $_.id } | Sort-Object id)
    if ($members.Count -ne @($Group.candidate_ids).Count) { Fail 'GROUP_MEMBERS_INVALID' }
    $packageId = 'live_galaxy_candidate_' + ($Group.id -replace '[^a-z0-9_]', '_')
    $buildId = "candidate-$($Group.id)-$($Group.build_profile_digest.Substring(0, 12))"
    $groupRoot = Join-Path $Destination $Group.id
    if (-not (Test-ContainedPath $groupRoot $Destination)) { Fail 'GROUP_PATH_ESCAPE' }
    $groupRoot = Resolve-NoReparseDestination $groupRoot
    if (-not (Test-ContainedPath $groupRoot $Destination)) { Fail 'GROUP_PATH_ESCAPE' }
    if (Test-Path -LiteralPath $groupRoot) { Fail 'GROUP_DESTINATION_EXISTS' }
    $null = New-Item -ItemType Directory -Path (Join-Path $groupRoot 'lua') -Force
    Set-OwnerOnly $groupRoot $true
    $null = New-Item -ItemType Directory -Path (Join-Path $groupRoot 'manifest') -Force
    $null = New-Item -ItemType Directory -Path (Join-Path $groupRoot 'tools/x4-verification/isolation') -Force
    $null = New-Item -ItemType Directory -Path (Join-Path $groupRoot 'tools/x4-verification/contracts') -Force

    $escapedPackageId = [Security.SecurityElement]::Escape($packageId)
    $escapedPackageName = [Security.SecurityElement]::Escape("Live Galaxy Candidate $($Group.id)")
    $escapedEntrypoint = [Security.SecurityElement]::Escape([string]$members[0].build_profile.entrypoint)
    $content = (Get-Content -LiteralPath $contentTemplatePath -Raw).Replace('{{PACKAGE_ID}}', $escapedPackageId).Replace('{{PACKAGE_NAME}}', $escapedPackageName)
    $ui = (Get-Content -LiteralPath $uiTemplatePath -Raw).Replace('{{PACKAGE_ID}}', $escapedPackageId).Replace('{{ENTRYPOINT}}', $escapedEntrypoint)
    Write-Utf8NoBom (Join-Path $groupRoot 'content.xml') $content
    Write-Utf8NoBom (Join-Path $groupRoot 'ui.xml') $ui
    Write-Utf8NoBom (Join-Path $groupRoot 'lua/live_galaxy_candidate_entry.lua') (New-Entrypoint $Group.id $buildId)
    Copy-Item -LiteralPath $runnerSourcePath -Destination (Join-Path $groupRoot 'lua/live_galaxy_candidate_runner.lua')
    Copy-Item -LiteralPath $adapterSourcePath -Destination (Join-Path $groupRoot 'tools/x4-verification/candidate-adapters.psm1')
    Copy-Item -LiteralPath $attestationModulePath -Destination (Join-Path $groupRoot 'tools/x4-verification/producer-attestation.psm1')
    Copy-Item -LiteralPath $workerSourcePath -Destination (Join-Path $groupRoot 'tools/x4-verification/isolation/candidate-worker.ps1')
    Copy-Item -LiteralPath $launcherSourcePath -Destination (Join-Path $groupRoot 'tools/x4-verification/isolation/invoke-candidate-worker.ps1')
    Copy-Item -LiteralPath $boundedReaderSourcePath -Destination (Join-Path $groupRoot 'tools/x4-verification/bounded-file.psm1')
    Copy-Item -LiteralPath $workerProtocolPath -Destination (Join-Path $groupRoot 'tools/x4-verification/contracts/candidate-worker-protocol.v1.json')
    Copy-Item -LiteralPath $runtimeEvidencePath -Destination (Join-Path $groupRoot 'tools/x4-verification/contracts/runtime-evidence.v1.json')
    Copy-Item -LiteralPath $ownerRootAnchorPath -Destination (Join-Path $groupRoot 'tools/x4-verification/contracts/owner-root-anchor.v1.json')

    $subset = [ordered]@{
        schema_version = $Matrix.schema_version
        matrix_id = $Matrix.matrix_id
        status = $Matrix.status
        evidence_classification = $Matrix.evidence_classification
        group = $Group
        candidates = $members
    }
    $subsetPath = Join-Path $groupRoot 'manifest/candidate-matrix-subset.v1.json'
    Write-CanonicalJson $subsetPath $subset
    $generatedContract = New-GeneratedConformanceContract $PackageContract $packageId $Group $members[0]
    $generatedContractPath = Join-Path $groupRoot 'manifest/package-conformance.v1.json'
    Write-CanonicalJson $generatedContractPath $generatedContract

    $conformance = Invoke-Conformance $groupRoot
    $requiredFiles = @($ManifestContract.required_generated_files)
    $generatedFiles = foreach ($logicalPath in $requiredFiles) {
        $physicalPath = Join-Path $groupRoot $logicalPath
        if (-not (Test-Path -LiteralPath $physicalPath -PathType Leaf)) { Fail 'GENERATED_FILE_MISSING' }
        $item = Get-Item -LiteralPath $physicalPath
        if ($item.Length -gt $ManifestContract.bounds.max_generated_file_bytes) { Fail 'GENERATED_FILE_BYTES_EXCEEDED' }
        [ordered]@{
            path = $logicalPath
            bytes = $item.Length
            sha256 = Get-FileDigest $physicalPath $ManifestContract.bounds.max_generated_file_bytes 'GENERATED_FILE_BYTES_EXCEEDED'
        }
    }
    [long]$generatedTotalBytes = 0
    foreach ($generatedFile in @($generatedFiles)) { $generatedTotalBytes += [long]$generatedFile.bytes }
    if (@($generatedFiles).Count -gt $ManifestContract.bounds.max_generated_files -or
        $generatedTotalBytes -gt $ManifestContract.bounds.max_generated_total_bytes) {
        Fail 'GENERATED_BOUNDS_EXCEEDED'
    }
    $packageConformance = [ordered]@{
        schema_version = $conformance.Value.schema_version
        verdict = $conformance.Value.verdict
        classification = $conformance.Value.classification
        evidence_level = $conformance.Value.evidence_level
        graph_digest = $conformance.Value.graph_digest
        dossier_digest = $conformance.Value.dossier_digest
        coverage_digest = $conformance.Value.coverage_digest
    }
    $packageConformanceBytes = [Text.Encoding]::UTF8.GetBytes(($packageConformance | ConvertTo-Json -Compress -Depth 32))
    $manifest = [ordered]@{
        schema_version = $ManifestContract.generated_schema_version
        build_id = $buildId
        group_id = $Group.id
        candidate_ids = @($Group.candidate_ids)
        developer_only = $true
        execution_status = 'execution-ready-local-process'
        native_execution_status = 'terminable-external-isolation'
        local_readiness_verified = $true
        dossier_digest = Get-FileDigest $dossierPath
        registry_digest = Get-FileDigest $registryPath
        coverage_digest = Get-FileDigest $coveragePath
        matrix_digest = $MatrixDigest
        build_profile_digest = $Group.build_profile_digest
        package_conformance_digest = Get-Sha256 $packageConformanceBytes
        runtime_evidence_schema_digest = Get-FileDigest $runtimeEvidencePath
        owner_root_anchor_digest = Get-FileDigest $ownerRootAnchorPath
        dispatcher_digest = Get-FileDigest $dispatcherSourcePath
        adapter_digest = Get-FileDigest $adapterSourcePath
        attestation_module_digest = Get-FileDigest $attestationModulePath
        bounded_reader_digest = Get-FileDigest $boundedReaderSourcePath
        worker_digest = Get-FileDigest $workerSourcePath
        launcher_digest = Get-FileDigest $launcherSourcePath
        worker_protocol_digest = Get-FileDigest $workerProtocolPath
        package_conformance = $packageConformance
        generated_files = @($generatedFiles)
    }
    Write-Json (Join-Path $groupRoot 'manifest/build-manifest.v1.json') $manifest
    foreach ($item in @(Get-ChildItem -LiteralPath $groupRoot -Recurse -Force | Sort-Object FullName -Descending)) {
        Set-OwnerOnly $item.FullName $item.PSIsContainer
    }
    $readinessOutput = Join-Path $Destination (".$($Group.id)-readiness.jsonl")
    $readiness = & pwsh -NoProfile -File $dispatcherSourcePath `
        -GroupRoot $groupRoot -OutputPath $readinessOutput 2>&1
    if ($LASTEXITCODE -ne 0) { Fail 'LOCAL_READINESS_FAILED' }
    try { $readinessResult = @($readiness)[-1] | ConvertFrom-Json -DateKind String }
    catch { Fail 'LOCAL_READINESS_RESULT_INVALID' }
    if ($readinessResult.local_process_ready -ne $true -or $readinessResult.evidence_classification -ne 'authenticated-local-contract') { Fail 'LOCAL_READINESS_FAILED' }
    Remove-Item -LiteralPath $readinessOutput -Force
    $script:createdGroups += $Group.id
}

function Write-Result([string]$Verdict, [string]$ReasonCode) {
    [ordered]@{
        schema_version = 'candidate-build-result.v1'
        verdict = $Verdict
        reason_code = $ReasonCode
        evidence_classification = 'scaffold-only'
        generated_group_ids = @($script:createdGroups | Sort-Object)
    } | ConvertTo-Json -Compress -Depth 8 | Write-Output
}

try {
    $destination = Resolve-SafeBuildRoot $BuildRoot
    $matrix = Read-Json $MatrixPath 'phase-05.1-candidates.v1'
    $manifestContract = Read-Json $manifestContractPath 'candidate-build-manifest-contract.v1'
    $packageContract = Read-Json $packageContractPath 'x4-package-conformance.v1'
    Assert-ValidMatrix $matrix
    $matrixDigest = Get-FileDigest $MatrixPath
    $null = New-Item -ItemType Directory -Path $destination -Force
    Set-OwnerOnly $destination $true
    foreach ($group in @($matrix.build_groups)) {
        New-GroupBuild $matrix $group $destination $manifestContract $packageContract $matrixDigest
    }
    Write-Result 'generated' 'GENERATED'
    exit 0
}
catch {
    Write-Verbose "$script:reasonCode at line $($_.InvocationInfo.ScriptLineNumber)"
    Write-Result 'rejected' $script:reasonCode
    exit 1
}
