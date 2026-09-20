$ErrorActionPreference = 'Stop'
$runner = Join-Path (Split-Path -Parent $PSScriptRoot) 'tools/run-heavy-ship-runtime.ps1'
if (-not (Test-Path -LiteralPath $runner -PathType Leaf)) {
    throw 'HEAVY_RUNTIME_RUNNER_MISSING'
}
& pwsh -NoProfile -File $runner -SelfTest
if ($LASTEXITCODE -ne 0) {
    throw "HEAVY_RUNTIME_RUNNER_SELF_TEST_FAILED:$LASTEXITCODE"
}
Write-Output 'heavy ship runtime runner tests passed'
