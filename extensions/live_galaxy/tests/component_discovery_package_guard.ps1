[CmdletBinding()]
param(
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$FixedProductionAllowlist = @(
    'extensions/live_galaxy/lua/live_galaxy_component_discovery.lua',
    'extensions/live_galaxy/lua/live_galaxy_x4_discovery.lua'
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
    if ($normalizedPath -notin $FixedProductionAllowlist) {
        return $false
    }

    foreach ($pattern in $AuthorityPatterns) {
        if ($Source -match $pattern) {
            return $false
        }
    }

    return $true
}

function Invoke-ComponentDiscoveryPackageGuardSelfTest {
    $allowlisted = @{
        ProductionPath = 'extensions/live_galaxy/lua/live_galaxy_component_discovery.lua'
        Source = 'local function read_station_count(api) return api.count_stations() end'
    }
    if (-not (Test-ComponentDiscoveryPackage @allowlisted)) {
        throw 'Allowlisted telemetry fixture was rejected.'
    }

    $expectedFailures = @(
        @{
            Name = 'authority vocabulary'
            ProductionPath = 'extensions/live_galaxy/lua/live_galaxy_x4_discovery.lua'
            Source = 'local function send_report() end'
        },
        @{
            Name = 'unallowlisted production path'
            ProductionPath = 'extensions/live_galaxy/lua/live_galaxy_runtime.lua'
            Source = 'local function read_station_count() end'
        },
        @{
            Name = 'source outside fixed allowlist'
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

if ($SelfTest -or $MyInvocation.InvocationName -ne '.') {
    Invoke-ComponentDiscoveryPackageGuardSelfTest
}
