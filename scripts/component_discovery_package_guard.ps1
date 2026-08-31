[CmdletBinding()]
param(
    [switch]$SelfTest,
    [string[]]$ProductionPath = @()
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$FixedProductionAllowlist = @(
    'extensions/live_galaxy/lua/live_galaxy_component_discovery.lua',
    'extensions/live_galaxy/lua/live_galaxy_x4_discovery.lua',
    'extensions/live_galaxy/lua/live_galaxy_runtime.lua',
    'extensions/live_galaxy/lua/live_galaxy_telemetry.lua'
)
$AuthorityPatterns = @(
    '(?i)(?<![A-Za-z0-9])report(?![A-Za-z0-9])',
    '(?i)(?<![A-Za-z0-9])acknowledg(?:e|ement)(?![A-Za-z0-9])',
    '(?i)(?<![A-Za-z0-9])command(?![A-Za-z0-9])',
    '(?i)(?<![A-Za-z0-9])effect(?![A-Za-z0-9])',
    '(?i)(?<![A-Za-z0-9])persistence(?![A-Za-z0-9])',
    '(?i)(?<![A-Za-z0-9])model(?![A-Za-z0-9])',
    '(?i)(?<![A-Za-z0-9])public[ _-]?ui(?![A-Za-z0-9])',
    '(?i)(?<![A-Za-z0-9])mutat(?:e|ion)(?![A-Za-z0-9])'
)

function Test-ComponentDiscoveryPackage {
    param(
        [Parameter(Mandatory)]
        [string]$ProductionPath,
        [Parameter(Mandatory)]
        [string]$Source
    )

    $normalizedPath = $ProductionPath.Replace('\', '/')
    if ($normalizedPath -notin $FixedProductionAllowlist) { return $false }
    foreach ($pattern in $AuthorityPatterns) {
        if ($Source -match $pattern) { return $false }
    }
    return $true
}

function Invoke-ComponentDiscoveryPackageGuardSelfTest {
    foreach ($path in $FixedProductionAllowlist) {
        if (-not (Test-ComponentDiscoveryPackage -ProductionPath $path `
                -Source 'local function read_station_count(api) return api.count_stations() end')) {
            throw "Allowlisted telemetry fixture was rejected: $path"
        }
    }
    $expectedFailures = @(
        @{
            Name = 'authority vocabulary'
            ProductionPath = $FixedProductionAllowlist[1]
            Source = 'local function send_report() end'
        },
        @{
            Name = 'unrelated Live Galaxy production path'
            ProductionPath = 'extensions/live_galaxy/lua/live_galaxy_normalize.lua'
            Source = 'local function read_station_count() end'
        },
        @{
            Name = 'foreign extension path'
            ProductionPath = 'extensions/other_package/lua/live_galaxy_component_discovery.lua'
            Source = 'local function read_station_count() end'
        }
    )
    foreach ($fixture in $expectedFailures) {
        if (Test-ComponentDiscoveryPackage -ProductionPath $fixture.ProductionPath -Source $fixture.Source) {
            throw "Expected failure fixture passed: $($fixture.Name)"
        }
    }
}

function Invoke-ComponentDiscoveryPackageGuard {
    param([Parameter(Mandatory)][string[]]$ProductionPaths)

    $root = Split-Path -Parent $PSScriptRoot
    foreach ($path in $ProductionPaths) {
        $normalizedPath = $path.Replace('\', '/')
        if ($normalizedPath -notin $FixedProductionAllowlist) {
            throw "Changed production path is outside the fixed allowlist: $normalizedPath"
        }
        $fullPath = Join-Path $root $normalizedPath
        if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
            throw "Allowlisted production file is missing: $normalizedPath"
        }
        $source = Get-Content -Raw -LiteralPath $fullPath
        if (-not (Test-ComponentDiscoveryPackage -ProductionPath $normalizedPath -Source $source)) {
            throw "Production package guard rejected: $normalizedPath"
        }
    }
}

if ($SelfTest) { Invoke-ComponentDiscoveryPackageGuardSelfTest }
if ($ProductionPath.Count -gt 0) {
    Invoke-ComponentDiscoveryPackageGuard -ProductionPaths $ProductionPath
}
