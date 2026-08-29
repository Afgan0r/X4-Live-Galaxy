param(
    [ValidateSet('all', 'x4_discovery', 'component_discovery', 'component_discovery_guard')]
    [string]$Suite = 'all',
    [string]$LuaPath
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))

if ($Suite -eq 'component_discovery_guard') {
    & powershell -NoProfile -File (Join-Path $PSScriptRoot 'component_discovery_package_guard.ps1') -SelfTest
    if ($LASTEXITCODE -ne 0) { throw 'Component discovery package guard failed.' }
    exit 0
}

if ($Suite -eq 'component_discovery') {
    & powershell -NoProfile -File (Join-Path $PSScriptRoot 'component_discovery_binding_contract.ps1')
    if ($LASTEXITCODE -ne 0) { throw 'Component discovery binding contract failed.' }
}

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

$tests = if ($Suite -eq 'x4_discovery') {
    @('x4_discovery_contract.lua')
} elseif ($Suite -eq 'component_discovery') {
    @('component_discovery_contract.lua')
} else {
    Get-ChildItem $PSScriptRoot -Filter '*_contract.lua' | ForEach-Object Name
}

foreach ($test in $tests) {
    $path = Join-Path $PSScriptRoot $test
    $modulePath = Join-Path $root 'extensions\?.lua'
    $moduleInitPath = Join-Path $root 'extensions\?\init.lua'
    $extensionLuaPath = Join-Path $root 'extensions\live_galaxy\lua\?.lua'
    & $lua -e "package.path = [[${modulePath};${moduleInitPath};${extensionLuaPath};]] .. package.path local cases = dofile([[${path}]]) for name, case in pairs(cases) do case() end"
    if ($LASTEXITCODE -ne 0) { throw "Lua contract failed: $test" }
}
