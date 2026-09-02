param(
    [ValidateSet('all', 'x4_discovery', 'component_discovery', 'component_discovery_guard', 'x4-package-conformance')]
    [string]$Suite = 'all',
    [string]$LuaPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$componentDiscoveryGuard = Join-Path $root 'scripts/component_discovery_package_guard.ps1'
$componentDiscoveryProductionPaths = @(
    'extensions/live_galaxy/lua/live_galaxy_component_discovery.lua',
    'extensions/live_galaxy/lua/live_galaxy_x4_discovery.lua',
    'extensions/live_galaxy/lua/live_galaxy_runtime.lua',
    'extensions/live_galaxy/lua/live_galaxy_telemetry.lua'
)

# One leaf-stage list is shared by focused selections and the aggregate.
$stages = @(
    @{ Name = 'package-conformance'; File = 'package_conformance_contract.ps1'; Suites = @('x4-package-conformance'); Arguments = @(); Failure = 'Product package conformance contract failed.' },
    @{ Name = 'component-binding'; File = 'component_discovery_binding_contract.ps1'; Suites = @('component_discovery'); Arguments = @(); Failure = 'Component discovery binding contract failed.' },
    @{ Name = 'component-guard-self-test'; File = $componentDiscoveryGuard; Suites = @('component_discovery', 'component_discovery_guard'); Arguments = @('-SelfTest'); Failure = 'Component discovery package guard self-test failed.' }
)
foreach ($productionPath in $componentDiscoveryProductionPaths) {
    $stages += @{
        Name = "component-guard:$productionPath"; File = $componentDiscoveryGuard
        Suites = @('component_discovery', 'component_discovery_guard')
        Arguments = @('-ProductionPath', $productionPath)
        Failure = "Component discovery package guard failed: $productionPath"
    }
}
$stages += @(
    @{ Name = 'component_discovery_contract.lua'; File = 'component_discovery_contract.lua'; Suites = @('component_discovery'); Arguments = @(); Failure = 'Lua contract failed: component_discovery_contract.lua' },
    @{ Name = 'x4_discovery_contract.lua'; File = 'x4_discovery_contract.lua'; Suites = @('x4_discovery'); Arguments = @(); Failure = 'Lua contract failed: x4_discovery_contract.lua' },
    @{ Name = 'telemetry_contract.lua'; File = 'telemetry_contract.lua'; Suites = @(); Arguments = @(); Failure = 'Lua contract failed: telemetry_contract.lua' },
    @{ Name = 'scheduler_contract.lua'; File = 'scheduler_contract.lua'; Suites = @(); Arguments = @(); Failure = 'Lua contract failed: scheduler_contract.lua' },
    @{ Name = 'persistence-schema'; File = 'persistence_schema_contract.ps1'; Suites = @(); Arguments = @(); Failure = 'Persistence schema contract failed.' },
    @{ Name = 'runner-process-results'; File = 'run_contracts_contract.ps1'; Suites = @(); Arguments = @(); Failure = 'Runner process-result contract failed.' }
)
$selected = @($stages | Where-Object { $Suite -eq 'all' -or $_.Suites -contains $Suite })
$lua = $null
if (@($selected | Where-Object { $_.File.EndsWith('.lua') }).Count -gt 0) {
    $lockPath = Join-Path $root 'tools/lua-runner.lock.json'
    if (-not (Test-Path -LiteralPath $lockPath -PathType Leaf)) {
        throw "Lua runner lock is missing: $lockPath"
    }
    $lock = Get-Content -LiteralPath $lockPath -Raw | ConvertFrom-Json
    if ([string]::IsNullOrWhiteSpace($lock.executableVersion) -or
        [string]::IsNullOrWhiteSpace($lock.materialization.executableRelativePath)) {
        throw 'Lua runner lock has required empty fields.'
    }

    function Test-LockedLua([string]$Candidate, [string]$Source) {
        if ([string]::IsNullOrWhiteSpace($Candidate) -or
            -not (Test-Path -LiteralPath $Candidate -PathType Leaf)) {
            return $null
        }

        $previousErrorActionPreference = $ErrorActionPreference
        try {
            $ErrorActionPreference = 'Continue'
            $reported = (& $Candidate -v 2>&1 | ForEach-Object {
                if ($_ -is [System.Management.Automation.ErrorRecord]) {
                    $_.Exception.Message
                }
                else {
                    $_.ToString()
                }
            } | Out-String).Trim()
        }
        finally {
            $ErrorActionPreference = $previousErrorActionPreference
        }
        if ($LASTEXITCODE -ne 0 -or
            $reported -notmatch "(?m)^$([regex]::Escape($lock.executableVersion))(?:\s|$)") {
            throw "Lua runner from $Source does not match lock '$($lock.executableVersion)': $reported"
        }
        return (Resolve-Path -LiteralPath $Candidate).Path
    }

    $lua = Test-LockedLua $LuaPath '-LuaPath'
    if ($null -eq $lua) {
        $lua = Test-LockedLua $env:LIVE_GALAXY_LUA 'LIVE_GALAXY_LUA'
    }
    if ($null -eq $lua) {
        $lua = Test-LockedLua (Join-Path $root $lock.materialization.executableRelativePath) 'project-local lock path'
    }
    if ($null -eq $lua) {
        $pathLua = Get-Command lua -ErrorAction SilentlyContinue
        if ($null -ne $pathLua) {
            $lua = Test-LockedLua $pathLua.Source 'PATH'
        }
    }
    if ($null -eq $lua) {
        throw "Lua runner unavailable. Set -LuaPath or LIVE_GALAXY_LUA, or run tools/provision-lua.ps1; every candidate must match '$($lock.executableVersion)'."
    }


}

foreach ($stage in $selected) {
    $path = if ([IO.Path]::IsPathRooted($stage.File)) { $stage.File } else { Join-Path $PSScriptRoot $stage.File }
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing product stage: $($stage.Name)" }
    $executable = 'pwsh'
    $arguments = @('-NoProfile', '-File', $path) + $stage.Arguments
    if ($stage.File.EndsWith('.lua')) {
        $executable = $lua
        $modulePath = Join-Path $root 'extensions/?.lua'
        $moduleInitPath = Join-Path $root 'extensions/?/init.lua'
        $extensionLuaPath = Join-Path $root 'extensions/live_galaxy/lua/?.lua'
        $test = $stage.File
        $luaCommand = @"
package.path = [[${modulePath};${moduleInitPath};${extensionLuaPath};]] .. package.path
local cases = dofile([[${path}]])
assert(type(cases) == 'table', 'MALFORMED_LUA_CASE_TABLE')
local count = 0
for name, case in pairs(cases) do
    assert(type(name) == 'string' and type(case) == 'function', 'MALFORMED_LUA_CASE_TABLE')
    count = count + 1
end
assert(count > 0, 'EMPTY_LUA_CASE_TABLE')
for name, case in pairs(cases) do
    case()
    print('PASS ${test}:' .. name)
end
print('CASES ${test}: ' .. count)
"@
        $arguments = @('-e', $luaCommand)
    }
    elseif ($stage.Name -eq 'runner-process-results') {
        $arguments += @('-LuaPath', $lua)
    }
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $exitCode = -1
    try {
        & $executable @arguments
        $exitCode = $LASTEXITCODE
        if ($exitCode -ne 0) { throw $stage.Failure }
    }
    finally {
        Write-Output ('STAGE {0} exit={1} elapsed_seconds={2:F3}' -f $stage.Name, $exitCode, $timer.Elapsed.TotalSeconds)
    }
}
