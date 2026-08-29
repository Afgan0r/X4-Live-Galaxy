param(
    [Parameter(Mandatory)]
    [ValidatePattern('^[A-Za-z0-9_-]{1,64}$')]
    [string]$AttemptId,
    [string]$ExtensionsRoot = 'F:\SteamLibrary\steamapps\common\X4 Foundations\extensions',
    [string]$EvidenceDirectory,
    [switch]$StartBridge
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if ([string]::IsNullOrWhiteSpace($EvidenceDirectory)) {
    $EvidenceDirectory = Join-Path $root 'runtime'
}
$installedExtension = Join-Path $ExtensionsRoot 'live_galaxy'
$installedConfig = Join-Path $installedExtension 'lua/live_galaxy_trace_config.lua'
$bridge = Join-Path $root 'target/debug/x4-bridge.exe'

if (Get-Process -Name 'X4' -ErrorAction SilentlyContinue) {
    throw 'refusing to change trace configuration while X4 is running'
}
if (-not (Test-Path $installedConfig)) { throw 'installed trace configuration is missing' }
if (-not (Test-Path $bridge)) { throw 'bridge binary is missing' }

New-Item -ItemType Directory -Force -Path $EvidenceDirectory | Out-Null
$traceConfig = @(
    'return {',
    '    enabled = true,',
    "    attempt_id = '$AttemptId',",
    '    max_frame_events = 64,',
    '}'
) -join [Environment]::NewLine
$utf8WithoutBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText($installedConfig, $traceConfig, $utf8WithoutBom)

$evidencePath = Join-Path $EvidenceDirectory "phase1-bridge-$AttemptId.log"
if ($StartBridge) {
    if (Get-Process -Name 'x4-bridge' -ErrorAction SilentlyContinue) {
        throw 'refusing to start a second bridge process'
    }
    $processInfo = New-Object System.Diagnostics.ProcessStartInfo
    $processInfo.FileName = $bridge
    $processInfo.WorkingDirectory = $root
    $processInfo.UseShellExecute = $false
    $processInfo.EnvironmentVariables['LIVE_GALAXY_DEBUG_EVIDENCE_PATH'] = $evidencePath
    $processInfo.EnvironmentVariables['LIVE_GALAXY_TRACE_ATTEMPT_ID'] = $AttemptId
    [System.Diagnostics.Process]::Start($processInfo) | Out-Null
}

[pscustomobject]@{
    AttemptId = $AttemptId
    BridgeEvidencePath = $evidencePath
    X4DebugLogMarker = "Live Galaxy runtime: attempt_id=$AttemptId"
    BridgeStarted = $StartBridge.IsPresent
}
