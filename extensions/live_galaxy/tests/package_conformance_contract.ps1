[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$checker = Join-Path $PSScriptRoot 'x4-package-conformance.ps1'
$packageRoot = Split-Path -Parent $PSScriptRoot
if (-not (Test-Path -LiteralPath $checker -PathType Leaf)) {
    throw 'Product package checker is missing.'
}
$output = @(& pwsh -NoProfile -File $checker -PackageRoot $packageRoot 2>&1)
$exitCode = $LASTEXITCODE
if ($exitCode -ne 0 -or $output -notcontains 'PASS package: live_galaxy (local static evidence)') {
    throw "Actual extension failed product package conformance: $($output -join [Environment]::NewLine)"
}
Write-Output 'PASS package fixture: actual-extension'
