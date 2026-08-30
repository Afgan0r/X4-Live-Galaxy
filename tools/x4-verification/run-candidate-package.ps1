[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateNotNullOrEmpty()][string]$GroupRoot,
    [Parameter(Mandatory)][ValidateNotNullOrEmpty()][string]$OutputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$launcherPath = Join-Path $repositoryRoot 'tools/x4-verification/isolation/invoke-candidate-worker.ps1'
$workerPath = Join-Path $repositoryRoot 'tools/x4-verification/isolation/candidate-worker.ps1'
$adapterPath = Join-Path $repositoryRoot 'tools/x4-verification/candidate-adapters.psm1'
$protocolPath = Join-Path $repositoryRoot 'tools/x4-verification/contracts/candidate-worker-protocol.v1.json'
$schemaPath = Join-Path $repositoryRoot 'tools/x4-verification/contracts/runtime-evidence.v1.json'
$script:ReasonCode = 'DISPATCH_INTERNAL_FAILURE'

function Fail([string]$Code) { $script:ReasonCode = $Code; throw [IO.InvalidDataException]::new($Code) }
function Get-Sha256([byte[]]$Bytes) { [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant() }
function Get-FileDigest([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { Fail 'COMPONENT_MISSING' }
    Get-Sha256 ([IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Path).Path))
}
function Test-Contained([string]$Path, [string]$Root) {
    $full = [IO.Path]::GetFullPath($Path).TrimEnd('\', '/')
    $base = [IO.Path]::GetFullPath($Root).TrimEnd('\', '/')
    $full.Equals($base, [StringComparison]::OrdinalIgnoreCase) -or $full.StartsWith($base + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)
}
function Assert-NoReparse([string]$Path) {
    $full = [IO.Path]::GetFullPath($Path)
    $root = [IO.Path]::GetPathRoot($full)
    $current = $root
    foreach ($segment in @($full.Substring($root.Length) -split '[\\/]+')) {
        if ([string]::IsNullOrEmpty($segment)) { continue }
        $current = Join-Path $current $segment
        if (-not (Test-Path -LiteralPath $current)) { break }
        if (((Get-Item -LiteralPath $current -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { Fail 'REPARSE_PATH_REJECTED' }
    }
}
function Set-OwnerOnly([string]$Path) {
    if (-not $IsWindows) {
        & chmod 600 -- $Path
        if ($LASTEXITCODE -ne 0) { Fail 'OUTPUT_PERMISSION_FAILED' }
        return
    }
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent().User
    $security = [Security.AccessControl.FileSecurity]::new()
    $security.SetOwner($identity)
    $security.SetAccessRuleProtection($true, $false)
    $rule = [Security.AccessControl.FileSystemAccessRule]::new(
        $identity,
        [Security.AccessControl.FileSystemRights]::Modify,
        [Security.AccessControl.AccessControlType]::Allow
    )
    $security.AddAccessRule($rule)
    Set-Acl -LiteralPath $Path -AclObject $security
}
function Write-Result([string]$Verdict, [bool]$Ready, [string]$Attestation, [string]$Digest = '') {
    [ordered]@{
        schema_version = 'candidate-package-run.v1'; verdict = $Verdict
        reason_code = $script:ReasonCode; local_process_ready = $Ready
        evidence_classification = 'authenticated-local-contract'
        retainable = $false; attestation_status = $Attestation; evidence_digest = $Digest
    } | ConvertTo-Json -Compress
}

$workRoot = $null
try {
    $groupFull = [IO.Path]::GetFullPath($GroupRoot)
    $outputFull = [IO.Path]::GetFullPath($OutputPath)
    Assert-NoReparse $groupFull
    Assert-NoReparse ([IO.Path]::GetDirectoryName($outputFull))
    if (-not (Test-Path -LiteralPath $groupFull -PathType Container) -or (Test-Path -LiteralPath $outputFull)) { Fail 'DISPATCH_PATH_INVALID' }
    $manifestPath = Join-Path $groupFull 'manifest/build-manifest.v1.json'
    $subsetPath = Join-Path $groupFull 'manifest/candidate-matrix-subset.v1.json'
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json -Depth 64 -DateKind String
    $subset = Get-Content -LiteralPath $subsetPath -Raw | ConvertFrom-Json -Depth 64 -DateKind String
    $candidateIds = @($subset.candidates.id | Sort-Object)
    if ($candidateIds.Count -lt 1 -or $candidateIds.Count -gt 7 -or
        ($candidateIds -join '|') -ne (@($manifest.candidate_ids | Sort-Object) -join '|')) { Fail 'CANDIDATE_SET_INVALID' }
    foreach ($file in @($manifest.generated_files)) {
        $candidatePath = Join-Path $groupFull ([string]$file.path)
        if (-not (Test-Contained $candidatePath $groupFull) -or (Get-FileDigest $candidatePath) -ne $file.sha256) { Fail 'COMPONENT_DIGEST_MISMATCH' }
    }
    Import-Module $adapterPath -Force
    $knownIds = @(Get-CandidateAdapterDefinitions).id
    if (@($candidateIds | Where-Object { $knownIds -cnotcontains $_ }).Count -ne 0) { Fail 'ADAPTER_ID_REJECTED' }

    $workRoot = Join-Path ([IO.Path]::GetTempPath()) ("live-galaxy-dispatch-" + [guid]::NewGuid().ToString('N'))
    $null = New-Item -ItemType Directory -Path $workRoot
    $runId = 'local-' + [guid]::NewGuid().ToString('N')
    $rows = @()
    foreach ($candidate in @($subset.candidates | Sort-Object id)) {
        $requestPath = Join-Path $workRoot "$($candidate.id).request.json"
        $responsePath = Join-Path $workRoot "$($candidate.id).response.json"
        $request = [ordered]@{
            schema_version = 'candidate-worker.v1'; request_id = [guid]::NewGuid().ToString('N')
            run_id = $runId; candidate_id = $candidate.id; adapter_id = 'local-contract-success'
            issued_at_utc = [DateTimeOffset]::UtcNow.ToString('O')
            input = [ordered]@{ expected_result = $candidate.expected_result; max_work_units = 8 }
        }
        [IO.File]::WriteAllText($requestPath, ($request | ConvertTo-Json -Compress -Depth 8), [Text.UTF8Encoding]::new($false))
        $launchText = & pwsh -NoProfile -File $launcherPath -RequestPath $requestPath -ResponsePath $responsePath -DeadlineMs 2000
        $launch = $launchText | ConvertFrom-Json -Depth 16 -DateKind String
        $execution = if ($launch.accepted) { 'pass' } else { 'fail' }
        $contract = if ($launch.accepted -and $launch.response.completeness -eq 'complete') { 'pass' } else { 'fail' }
        $effect = if ($contract -eq 'pass' -and $launch.response.actual_result -ceq $candidate.expected_result) { 'pass' } else { 'mismatch' }
        $rows += [ordered]@{
            schema_version = 'candidate-local-evidence.v1'; run_id = $runId; candidate_id = $candidate.id
            build_id = $manifest.build_id; group_id = $manifest.group_id; execution_verdict = $execution
            contract_verdict = $contract; effect_verdict = $effect; completeness = if ($launch.accepted) { $launch.response.completeness } else { 'incomplete' }
            diagnostic_code = $launch.diagnostic_code; evidence_classification = 'authenticated-local-contract'
        }
    }
    if (@($rows | Where-Object { $_.execution_verdict -ne 'pass' -or $_.contract_verdict -ne 'pass' -or $_.effect_verdict -ne 'pass' }).Count -ne 0) { Fail 'CANDIDATE_RUN_INCOMPLETE' }
    $text = (@($rows | ForEach-Object { $_ | ConvertTo-Json -Compress -Depth 8 }) -join "`n") + "`n"
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($text)
    if ($bytes.Length -gt 65536) { Fail 'OUTPUT_BOUND_EXCEEDED' }
    $temporary = "$outputFull.$PID.tmp"
    [IO.File]::WriteAllBytes($temporary, $bytes)
    Set-OwnerOnly $temporary
    [IO.File]::Move($temporary, $outputFull)
    $digest = Get-FileDigest $outputFull
    $script:ReasonCode = 'PRODUCER_ATTESTATION_UNCONFIGURED'
    Write-Result 'pass' $true 'PRODUCER_ATTESTATION_UNCONFIGURED' $digest
    exit 0
}
catch {
    Write-Result 'fail' $false 'PRODUCER_ATTESTATION_UNCONFIGURED'
    exit 1
}
finally {
    if ($null -ne $workRoot -and (Test-Path -LiteralPath $workRoot)) { Remove-Item -LiteralPath $workRoot -Recurse -Force }
}
