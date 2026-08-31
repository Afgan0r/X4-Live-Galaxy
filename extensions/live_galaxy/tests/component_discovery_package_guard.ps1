[CmdletBinding()]
param(
    [switch]$SelfTest,
    [string[]]$ProductionPath = @()
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$guard = Join-Path $root 'scripts/component_discovery_package_guard.ps1'
if (-not (Test-Path -LiteralPath $guard -PathType Leaf)) {
    throw "Authoritative component discovery package guard is missing: $guard"
}
& $guard @PSBoundParameters
