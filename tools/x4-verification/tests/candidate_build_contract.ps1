[CmdletBinding()]
param(
    [ValidateSet('partition', 'generated-builds', 'all')]
    [string]$Case = 'all'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$matrixPath = Join-Path $root 'tests/x4-candidates/phase-05.1-candidates.v1.json'
$manifestContractPath = Join-Path $root 'tools/x4-verification/contracts/candidate-build-manifest.v1.json'
$builderPath = Join-Path $root 'tools/x4-verification/build-candidate-extension.ps1'
$contentTemplatePath = Join-Path $root 'tools/x4-verification/templates/candidate-content.xml'
$uiTemplatePath = Join-Path $root 'tools/x4-verification/templates/candidate-ui.xml'
$runtimeEvidencePath = Join-Path $root 'tools/x4-verification/contracts/runtime-evidence.v1.json'
$dossierPath = Join-Path $root 'tools/x4-verification/contracts/phase-05.1-dossier.v1.json'
$registryPath = Join-Path $root 'tools/x4-verification/contracts/known-failures.v1.json'
$coveragePath = Join-Path $root 'tools/x4-verification/contracts/coverage.v1.json'
$packageContractPath = Join-Path $root 'tools/x4-verification/contracts/package-conformance.v1.json'
$publicPackageRoot = Join-Path $root 'extensions/live_galaxy'

$expectedCandidateIds = @(
    'p051-cadence-seta',
    'p051-lifecycle-reload',
    'p051-mod-stack-compatibility',
    'p051-native-count-fill-runtime',
    'p051-native-fill-completeness',
    'p051-native-identity-closure',
    'p051-native-volume-envelope'
)
$profileFields = @('content_profile', 'ui_registration_profile', 'entrypoint', 'import_root', 'binding_profile')
$verdictAxes = @('execution', 'contract', 'effect')
$sourceExclusions = @(
    'equate-frame-budget-with-native-allocation-bound',
    'invent-native-offset-pagination',
    'select-production-ceiling-from-one-run',
    'treat-emission-paging-as-discovery-paging',
    'truncate-owner-scope'
)

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Get-Sha256([byte[]]$Bytes) {
    return [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant()
}

function Get-FileDigest([string]$Path) {
    return Get-Sha256 ([IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Path).Path))
}

function Get-ProfileDigest($Profile) {
    $lines = foreach ($field in $profileFields) {
        Assert-True ($null -ne $Profile.PSObject.Properties[$field]) "Build profile is missing '$field'."
        $value = [string]$Profile.$field
        Assert-True (-not [string]::IsNullOrWhiteSpace($value)) "Build profile '$field' is empty."
        "$field=$value"
    }
    Assert-True (@($Profile.PSObject.Properties).Count -eq $profileFields.Count) 'Build profile contains undeclared fields.'
    return Get-Sha256 ([Text.Encoding]::UTF8.GetBytes(($lines -join "`n")))
}

function Read-Json([string]$Path) {
    Assert-True (Test-Path -LiteralPath $Path -PathType Leaf) "Missing contract artifact: $Path"
    return Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json -Depth 64
}

function Copy-JsonValue($Value) {
    return ($Value | ConvertTo-Json -Depth 64 | ConvertFrom-Json -Depth 64)
}

function New-DirectoryReparsePoint([string]$Path, [string]$Target) {
    $itemType = if ($IsWindows) { 'Junction' } else { 'SymbolicLink' }
    $null = New-Item -ItemType $itemType -Path $Path -Target $Target
}

function Test-StringArray($Values, [int]$Minimum, [int]$Maximum, [string]$Name) {
    $array = @($Values)
    Assert-True ($array.Count -ge $Minimum -and $array.Count -le $Maximum) "$Name count is invalid."
    foreach ($value in $array) {
        Assert-True ($value -is [string] -and -not [string]::IsNullOrWhiteSpace($value) -and $value.Length -le 256) "$Name contains an invalid value."
    }
    Assert-True (@($array | Sort-Object -Unique).Count -eq $array.Count) "$Name contains duplicates."
}

function Assert-ValidMatrix($Matrix) {
    Assert-True ($Matrix.schema_version -eq 'phase-05.1-candidates.v1') 'Unexpected matrix schema.'
    Assert-True ($Matrix.status -eq 'runtime-pending') 'Matrix must remain runtime-pending.'
    Assert-True ($Matrix.evidence_classification -eq 'scaffold-only') 'Matrix must remain scaffold-only until runtime adapters exist.'
    Assert-True ($Matrix.bounds.max_candidates -eq 7) 'Matrix must be bounded to the exact seven candidates.'
    Assert-True (@($Matrix.candidates).Count -eq 7) 'Matrix must contain exactly seven candidates.'
    Assert-True ((@($Matrix.candidates.id) -join '|') -eq ($expectedCandidateIds -join '|')) 'Candidates are missing, duplicated, or not sorted.'
    Assert-True ((@($Matrix.build_groups.id) -join '|') -eq ((@($Matrix.build_groups.id) | Sort-Object) -join '|')) 'Build groups are not sorted.'
    Assert-True ((@($Matrix.source_resolvable_exclusions) -join '|') -eq ($sourceExclusions -join '|')) 'Source-resolvable exclusions are incomplete or unstable.'

    $sourceInputs = @{
        dossier = $dossierPath
        registry = $registryPath
        coverage = $coveragePath
        runtime_evidence = $runtimeEvidencePath
        package_conformance = $packageContractPath
    }
    foreach ($name in $sourceInputs.Keys) {
        $reference = @($Matrix.source_inputs | Where-Object { $_.id -eq $name })
        Assert-True ($reference.Count -eq 1) "Matrix source input '$name' is missing or duplicated."
        Assert-True ($reference[0].sha256 -eq (Get-FileDigest $sourceInputs[$name])) "Matrix source input '$name' digest is stale."
    }

    $byId = @{}
    foreach ($candidate in @($Matrix.candidates)) {
        Assert-True (-not $byId.ContainsKey($candidate.id)) "Duplicate candidate '$($candidate.id)'."
        $byId[$candidate.id] = $candidate
        Assert-True ($candidate.status -eq 'runtime-pending') "Candidate '$($candidate.id)' is not runtime-pending."
        Assert-True ($candidate.source_action_only -eq $false) "Candidate '$($candidate.id)' is source-action-only."
        Assert-True ($candidate.build_profile_digest -eq (Get-ProfileDigest $candidate.build_profile)) "Candidate '$($candidate.id)' profile digest is invalid."
        Test-StringArray $candidate.evidence_ids 1 16 "Candidate '$($candidate.id)' evidence IDs"
        Test-StringArray $candidate.failure_classifications 1 8 "Candidate '$($candidate.id)' failure classifications"
        Test-StringArray $candidate.bounded_steps 1 8 "Candidate '$($candidate.id)' bounded steps"
        Test-StringArray $candidate.stop_conditions 1 8 "Candidate '$($candidate.id)' stop conditions"
        Assert-True ((@($candidate.verdict_axes) -join '|') -eq ($verdictAxes -join '|')) "Candidate '$($candidate.id)' verdict axes are invalid."
        Assert-True (-not [string]::IsNullOrWhiteSpace([string]$candidate.expected_result)) "Candidate '$($candidate.id)' expected result is missing."
        Test-StringArray $candidate.conflicts_with 0 6 "Candidate '$($candidate.id)' conflicts"
        Assert-True (-not (@($candidate.conflicts_with) -contains $candidate.id)) "Candidate '$($candidate.id)' conflicts with itself."
    }

    foreach ($candidate in @($Matrix.candidates)) {
        foreach ($conflictId in @($candidate.conflicts_with)) {
            Assert-True ($byId.ContainsKey($conflictId)) "Candidate '$($candidate.id)' has an undeclared conflict target."
            Assert-True (@($byId[$conflictId].conflicts_with) -contains $candidate.id) "Candidate '$($candidate.id)' has an asymmetric conflict."
        }
    }

    $groupById = @{}
    $membershipCounts = @{}
    foreach ($candidateId in $expectedCandidateIds) { $membershipCounts[$candidateId] = 0 }
    foreach ($group in @($Matrix.build_groups)) {
        Assert-True (-not $groupById.ContainsKey($group.id)) "Duplicate group '$($group.id)'."
        $groupById[$group.id] = $group
        $members = @($group.candidate_ids)
        Assert-True ($members.Count -gt 0) "Group '$($group.id)' is empty."
        Assert-True (($members -join '|') -eq (($members | Sort-Object) -join '|')) "Group '$($group.id)' members are not sorted."
        Assert-True ($group.build_profile_digest -match '^[a-f0-9]{64}$') "Group '$($group.id)' digest is invalid."
        foreach ($memberId in $members) {
            Assert-True ($byId.ContainsKey($memberId)) "Group '$($group.id)' contains an undeclared candidate."
            $membershipCounts[$memberId] = [int]$membershipCounts[$memberId] + 1
            $candidate = $byId[$memberId]
            Assert-True ($candidate.build_group -eq $group.id) "Candidate '$memberId' group linkage is invalid."
            Assert-True ($candidate.build_profile_digest -eq $group.build_profile_digest) "Group '$($group.id)' mixes profiles."
        }
    }

    foreach ($candidate in @($Matrix.candidates)) {
        Assert-True ($groupById.ContainsKey($candidate.build_group)) "Candidate '$($candidate.id)' references a missing group."
        $groupMembers = @($groupById[$candidate.build_group].candidate_ids)
        Assert-True ($membershipCounts[$candidate.id] -eq 1 -and $groupMembers -contains $candidate.id) "Candidate '$($candidate.id)' must occur exactly once in its declared group."
        if ($candidate.exclusive_build -eq $true) {
            Assert-True ($groupMembers.Count -eq 1) "Exclusive candidate '$($candidate.id)' shares a group."
            foreach ($other in @($Matrix.candidates | Where-Object { $_.id -ne $candidate.id })) {
                Assert-True (@($candidate.conflicts_with) -contains $other.id) "Exclusive candidate '$($candidate.id)' has an undeclared conflict."
            }
        }
        foreach ($other in @($Matrix.candidates | Where-Object { $_.id -gt $candidate.id })) {
            $conflict = @($candidate.conflicts_with) -contains $other.id
            $shareable = $candidate.build_profile_digest -eq $other.build_profile_digest -and
                $candidate.exclusive_build -ne $true -and $other.exclusive_build -ne $true -and -not $conflict
            if ($shareable) {
                Assert-True ($candidate.build_group -eq $other.build_group) "Safe peers '$($candidate.id)' and '$($other.id)' were not grouped."
            }
            if ($candidate.build_group -eq $other.build_group) {
                Assert-True ($shareable) "Conflicting or incompatible candidates share '$($candidate.build_group)'."
            }
        }
    }
}

function Assert-Rejected([scriptblock]$Mutation, [string]$Name) {
    $copy = Copy-JsonValue (Read-Json $matrixPath)
    & $Mutation $copy
    $rejected = $false
    try { Assert-ValidMatrix $copy }
    catch { $rejected = $true }
    Assert-True $rejected "Negative matrix '$Name' did not fail closed."
}

function Test-Partition {
    $matrix = Read-Json $matrixPath
    $manifestContract = Read-Json $manifestContractPath
    Assert-ValidMatrix $matrix
    Assert-True ($manifestContract.schema_version -eq 'candidate-build-manifest-contract.v1') 'Unexpected build-manifest contract schema.'
    Assert-True ((@($manifestContract.required_fields) -contains 'package_conformance_digest')) 'Build manifest does not require package conformance linkage.'
    Assert-True ((@($manifestContract.required_digests) -join '|') -eq 'dossier_digest|registry_digest|coverage_digest|matrix_digest|build_profile_digest|package_conformance_digest|runtime_evidence_schema_digest|owner_root_anchor_digest|dispatcher_digest|adapter_digest|attestation_module_digest|bounded_reader_digest|worker_digest|launcher_digest|worker_protocol_digest') 'Build manifest digest chain is incomplete or unstable.'

    Assert-Rejected { param($m) $m.candidates = @() } 'empty'
    Assert-Rejected { param($m) $m.candidates += Copy-JsonValue $m.candidates[0] } 'duplicate'
    Assert-Rejected { param($m) $m.candidates = @($m.candidates | Select-Object -Skip 1) } 'missing'
    Assert-Rejected { param($m) $m.candidates[0].source_action_only = $true } 'source-action-only'
    Assert-Rejected { param($m) $m.candidates[0].conflicts_with = @('missing-candidate') } 'undeclared-conflict'
    Assert-Rejected { param($m) $m.candidates[0].conflicts_with = @() } 'asymmetric-conflict'
    Assert-Rejected { param($m) $m.candidates[0].build_profile.binding_profile = 'changed-binding' } 'profile-digest-mismatch'
    Assert-Rejected { param($m) $m.candidates[0].build_group = 'p051-build-lifecycle' } 'conflicting-same-group'
    Assert-Rejected { param($m) $m.build_groups[1].candidate_ids = @($m.build_groups[1].candidate_ids | Select-Object -Skip 1) } 'candidate-missing-from-groups'
    Assert-Rejected { param($m) $m.build_groups[1].candidate_ids = @($m.build_groups[1].candidate_ids[0]) + @($m.build_groups[1].candidate_ids) } 'candidate-duplicated-in-group'
    Assert-Rejected { param($m) $m.build_groups[0].candidate_ids = @($m.build_groups[0].candidate_ids) + @($m.build_groups[1].candidate_ids[0]) } 'candidate-in-wrong-group'
}

function Invoke-Builder([string]$BuildRoot, [int]$ExpectedExitCode, [string]$CandidateMatrixPath = $matrixPath) {
    $output = & pwsh -NoProfile -File $builderPath -BuildRoot $BuildRoot -MatrixPath $CandidateMatrixPath 2>&1
    $exitCode = $LASTEXITCODE
    Assert-True ($exitCode -eq $ExpectedExitCode) "Builder exit $exitCode, expected ${ExpectedExitCode}: $($output -join [Environment]::NewLine)"
    return @($output)
}

function Assert-Manifest([string]$GroupRoot, $Matrix, $Group) {
    $manifestPath = Join-Path $GroupRoot 'manifest/build-manifest.v1.json'
    $manifest = Read-Json $manifestPath
    $contract = Read-Json $manifestContractPath
    Assert-True ($manifest.schema_version -eq $contract.generated_schema_version) 'Generated manifest schema is invalid.'
    foreach ($field in @($contract.required_fields)) {
        Assert-True ($null -ne $manifest.PSObject.Properties[$field]) "Generated manifest is missing '$field'."
    }
    foreach ($field in @($contract.required_digests)) {
        Assert-True ([string]$manifest.$field -match '^[a-f0-9]{64}$') "Generated manifest digest '$field' is invalid."
    }
    Assert-True ($manifest.group_id -eq $Group.id) 'Generated manifest group identity is invalid.'
    Assert-True ((@($manifest.candidate_ids) -join '|') -eq (@($Group.candidate_ids) -join '|')) 'Generated manifest candidate membership is invalid.'
    Assert-True ($manifest.build_profile_digest -eq $Group.build_profile_digest) 'Generated manifest profile linkage is invalid.'
    Assert-True ($manifest.matrix_digest -eq (Get-FileDigest $matrixPath)) 'Generated manifest matrix linkage is stale.'
    Assert-True ($manifest.runtime_evidence_schema_digest -eq (Get-FileDigest $runtimeEvidencePath)) 'Generated manifest evidence-schema linkage is stale.'
    Assert-True ($manifest.owner_root_anchor_digest -eq (Get-FileDigest (Join-Path $root 'tools/x4-verification/contracts/owner-root-anchor.v1.json'))) 'Generated manifest owner-root linkage is stale.'
    Assert-True ($manifest.developer_only -eq $true -and $manifest.execution_status -eq 'execution-ready-local-process') 'Generated manifest is not locally execution-ready.'
    Assert-True ($manifest.native_execution_status -eq 'terminable-external-isolation') 'Generated manifest does not bind the terminable external worker.'
    Assert-True ($manifest.local_readiness_verified -eq $true) 'Generated manifest copied readiness text without executing the local contract.'
    $entrypoint = Get-Content -LiteralPath (Join-Path $GroupRoot 'lua/live_galaxy_candidate_entry.lua') -Raw
    Assert-True ($entrypoint -match 'execution_ready_local_process = true') 'Generated entrypoint omits local-process readiness metadata.'
    Assert-True ($entrypoint -notmatch 'ffi\.C') 'Generated entrypoint executes or exposes direct native access.'
    Assert-True ($entrypoint -notmatch '(?i)dispatch|execute|launch|save|mutation') 'Generated entrypoint exposes a runtime control.'
    foreach ($requiredPath in @(
        'tools/x4-verification/candidate-adapters.psm1',
        'tools/x4-verification/producer-attestation.psm1',
        'tools/x4-verification/bounded-file.psm1',
        'tools/x4-verification/isolation/candidate-worker.ps1',
        'tools/x4-verification/isolation/invoke-candidate-worker.ps1',
        'tools/x4-verification/contracts/candidate-worker-protocol.v1.json',
        'tools/x4-verification/contracts/runtime-evidence.v1.json'
    )) {
        Assert-True (@($manifest.generated_files.path) -contains $requiredPath) "Generated root omits runtime component '$requiredPath'."
    }
    $generatedPaths = @($manifest.generated_files.path)
    Assert-True (($generatedPaths | Sort-Object -Unique).Count -eq $generatedPaths.Count) 'Generated file manifest contains duplicates.'
    foreach ($file in @($manifest.generated_files)) {
        $path = Join-Path $GroupRoot $file.path
        Assert-True (Test-Path -LiteralPath $path -PathType Leaf) "Generated file '$($file.path)' is missing."
        Assert-True ($file.sha256 -eq (Get-FileDigest $path)) "Generated file '$($file.path)' digest is stale."
    }
    $serialized = $manifest | ConvertTo-Json -Depth 64 -Compress
    Assert-True ($serialized -notmatch '(?i)game[_-]?launch|launch[_-]?x4|save[_-]?access|game[_-]?mutation|acknowledg|player[_-]?report') 'Generated manifest exposes a prohibited control.'
}

function Test-GeneratedBuilds {
    foreach ($required in @($builderPath, $contentTemplatePath, $uiTemplatePath, $matrixPath, $manifestContractPath)) {
        Assert-True (Test-Path -LiteralPath $required -PathType Leaf) "Missing build artifact: $required"
    }
    $matrix = Read-Json $matrixPath
    Assert-ValidMatrix $matrix
    $scratch = Join-Path ([IO.Path]::GetTempPath()) ('live-galaxy-candidate-build-' + [guid]::NewGuid().ToString('N'))
    $reparseRoot = Join-Path $scratch 'reparse-root'
    try {
        $reparseTarget = Join-Path $scratch 'reparse-target'
        $null = New-Item -ItemType Directory -Path $reparseTarget -Force
        New-DirectoryReparsePoint $reparseRoot $reparseTarget
        $null = Invoke-Builder $reparseRoot 1
        Assert-True (@(Get-ChildItem -LiteralPath $reparseTarget -Force).Count -eq 0) 'Builder wrote through a destination reparse point.'
        [IO.Directory]::Delete($reparseRoot)
        [IO.Directory]::Delete($reparseTarget)

        $null = Invoke-Builder $scratch 0
        $roots = @(Get-ChildItem -LiteralPath $scratch -Directory | Sort-Object Name)
        Assert-True ($roots.Count -eq @($matrix.build_groups).Count) 'Builder did not create exactly one root per group.'
        $seen = @{}
        foreach ($group in @($matrix.build_groups)) {
            $groupRoot = Join-Path $scratch $group.id
            Assert-True (Test-Path -LiteralPath $groupRoot -PathType Container) "Group root '$($group.id)' is missing."
            Assert-Manifest $groupRoot $matrix $group
            foreach ($candidateId in @($group.candidate_ids)) {
                Assert-True (-not $seen.ContainsKey($candidateId)) "Candidate '$candidateId' appears in multiple roots."
                $seen[$candidateId] = $true
            }
        }
        Assert-True ($seen.Count -eq 7) 'Generated roots do not cover exactly seven candidates.'

        $firstGroup = @($matrix.build_groups)[0]
        $firstRoot = Join-Path $scratch $firstGroup.id
        $manifestPath = Join-Path $firstRoot 'manifest/build-manifest.v1.json'
        $originalManifest = Get-Content -LiteralPath $manifestPath -Raw
        try {
            $tamperedManifest = $originalManifest | ConvertFrom-Json -Depth 64
            $tamperedManifest.matrix_digest = ('0' * 64)
            Set-Content -LiteralPath $manifestPath -Value ($tamperedManifest | ConvertTo-Json -Depth 64) -NoNewline -Encoding utf8
            $rejected = $false
            try { Assert-Manifest $firstRoot $matrix $firstGroup }
            catch { $rejected = $true }
            Assert-True $rejected 'Tampered generated manifest did not fail closed.'
        }
        finally {
            Set-Content -LiteralPath $manifestPath -Value $originalManifest -NoNewline -Encoding utf8
        }

        $tamperedMatrixPath = Join-Path $scratch 'tampered-matrix.json'
        $tamperedMatrix = Copy-JsonValue $matrix
        $tamperedMatrix.candidates[0].build_profile_digest = ('0' * 64)
        Set-Content -LiteralPath $tamperedMatrixPath -Value ($tamperedMatrix | ConvertTo-Json -Depth 64) -NoNewline -Encoding utf8
        $tamperedRoot = Join-Path $scratch 'tampered-output'
        $null = Invoke-Builder $tamperedRoot 1 $tamperedMatrixPath
        Assert-True (-not (Test-Path -LiteralPath $tamperedRoot)) 'Builder wrote output for a stale profile digest.'

        $requiredFields = @('question', 'expected_result', 'evidence_ids', 'failure_classifications', 'bounded_steps', 'stop_conditions', 'verdict_axes')
        foreach ($field in $requiredFields) {
            $invalid = Copy-JsonValue $matrix
            $invalid.candidates[0].PSObject.Properties.Remove($field)
            $invalidPath = Join-Path $scratch "missing-$field.json"
            Set-Content -LiteralPath $invalidPath -Value ($invalid | ConvertTo-Json -Depth 64) -NoNewline -Encoding utf8
            $null = Invoke-Builder (Join-Path $scratch "reject-missing-$field") 1 $invalidPath
        }
        $missingExclusions = Copy-JsonValue $matrix
        $missingExclusions.source_resolvable_exclusions = @($missingExclusions.source_resolvable_exclusions | Select-Object -Skip 1)
        $missingExclusionsPath = Join-Path $scratch 'missing-exclusions.json'
        Set-Content -LiteralPath $missingExclusionsPath -Value ($missingExclusions | ConvertTo-Json -Depth 64) -NoNewline -Encoding utf8
        $null = Invoke-Builder (Join-Path $scratch 'reject-missing-exclusions') 1 $missingExclusionsPath

        $membershipMutations = @(
            @{ Name = 'missing-member'; Apply = { param($m) $m.build_groups[1].candidate_ids = @($m.build_groups[1].candidate_ids | Select-Object -Skip 1) } },
            @{ Name = 'duplicate-member'; Apply = { param($m) $m.build_groups[1].candidate_ids = @($m.build_groups[1].candidate_ids[0]) + @($m.build_groups[1].candidate_ids) } },
            @{ Name = 'wrong-group-member'; Apply = { param($m) $m.build_groups[0].candidate_ids = @($m.build_groups[0].candidate_ids) + @($m.build_groups[1].candidate_ids[0]) } }
        )
        foreach ($mutation in $membershipMutations) {
            $invalidMembership = Copy-JsonValue $matrix
            & $mutation.Apply $invalidMembership
            $invalidMembershipPath = Join-Path $scratch "$($mutation.Name).json"
            Set-Content -LiteralPath $invalidMembershipPath -Value ($invalidMembership | ConvertTo-Json -Depth 64) -NoNewline -Encoding utf8
            $invalidMembershipRoot = Join-Path $scratch "reject-$($mutation.Name)"
            $null = Invoke-Builder $invalidMembershipRoot 1 $invalidMembershipPath
            Assert-True (-not (Test-Path -LiteralPath $invalidMembershipRoot)) "Builder wrote output for '$($mutation.Name)'."
        }

        $invalidGroup = Copy-JsonValue $matrix
        $invalidGroup.build_groups[0].id = 'group"; os.execute("unsafe") --'
        $invalidGroup.candidates[0].build_group = $invalidGroup.build_groups[0].id
        $invalidGroupPath = Join-Path $scratch 'reject-injected-group.json'
        Set-Content -LiteralPath $invalidGroupPath -Value ($invalidGroup | ConvertTo-Json -Depth 64) -NoNewline -Encoding utf8
        $invalidGroupRoot = Join-Path $scratch 'reject-injected-group'
        $null = Invoke-Builder $invalidGroupRoot 1 $invalidGroupPath
        Assert-True (-not (Test-Path -LiteralPath $invalidGroupRoot)) 'Builder wrote output for an injected group ID.'

        $before = Get-FileDigest (Join-Path $publicPackageRoot 'content.xml')
        $null = Invoke-Builder $publicPackageRoot 1
        Assert-True ((Get-FileDigest (Join-Path $publicPackageRoot 'content.xml')) -eq $before) 'Builder changed the public package after rejecting it.'
        $null = Invoke-Builder $root 1
        $null = Invoke-Builder ([IO.Path]::GetPathRoot($root)) 1
        foreach ($forbiddenRoot in @(
            (Join-Path $scratch 'steamapps/common/X4 Foundations/extensions/live_galaxy'),
            (Join-Path $scratch 'staging/extensions/live_galaxy'),
            (Join-Path $scratch 'Documents/Egosoft/X4/123456/save/builds')
        )) {
            $null = Invoke-Builder $forbiddenRoot 1
            Assert-True (-not (Test-Path -LiteralPath $forbiddenRoot)) `
                "Builder created a forbidden destination: $forbiddenRoot"
        }
        $gameRoot = 'F:\SteamLibrary\steamapps\common\X4 Foundations'
        if (Test-Path -LiteralPath $gameRoot -PathType Container) {
            $null = Invoke-Builder $gameRoot 1
        }
    }
    finally {
        if (Test-Path -LiteralPath $reparseRoot) { [IO.Directory]::Delete($reparseRoot) }
        if (Test-Path -LiteralPath $scratch) {
            $resolved = [IO.Path]::GetFullPath($scratch)
            $temp = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
            Assert-True ($resolved.StartsWith($temp, [StringComparison]::OrdinalIgnoreCase)) 'Refusing broad fixture cleanup.'
            Remove-Item -LiteralPath $scratch -Recurse -Force
        }
    }
}

switch ($Case) {
    'partition' { Test-Partition }
    'generated-builds' { Test-GeneratedBuilds }
    'all' { Test-Partition; Test-GeneratedBuilds }
}

Write-Output "candidate build contract passed: $Case"
