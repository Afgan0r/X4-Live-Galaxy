param(
    [Parameter(Mandatory)]
    [ValidatePattern('^[A-Za-z0-9_-]{1,64}$')]
    [string]$AttemptId,
    [string]$DebugLog = 'C:\Users\pavlo\Documents\Egosoft\X4\128881100\debug.log',
    [string]$EvidenceDirectory
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if ([string]::IsNullOrWhiteSpace($EvidenceDirectory)) {
    $EvidenceDirectory = Join-Path $root 'runtime'
}
if (-not (Test-Path $DebugLog)) { throw 'X4 debug.log is missing' }

$marker = "Live Galaxy runtime: attempt_id=$AttemptId"
$lines = @(Select-String -LiteralPath $DebugLog -SimpleMatch $marker | ForEach-Object Line)
if ($lines.Count -eq 0) { throw 'no matching Live Galaxy trace lines were found' }

New-Item -ItemType Directory -Force -Path $EvidenceDirectory | Out-Null
$output = Join-Path $EvidenceDirectory "phase1-x4-$AttemptId.log"
$utf8WithoutBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllLines($output, [string[]]$lines, $utf8WithoutBom)

[pscustomobject]@{
    AttemptId = $AttemptId
    X4TracePath = $output
    CapturedLines = $lines.Count
}
