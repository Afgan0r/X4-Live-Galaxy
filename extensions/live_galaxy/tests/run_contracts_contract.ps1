[CmdletBinding()]
param([string]$LuaPath)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$runner = Join-Path $PSScriptRoot 'run_contracts.ps1'
$pwsh = (Get-Command pwsh -ErrorAction Stop).Source
if (-not (Test-Path -LiteralPath $runner -PathType Leaf)) { throw 'Runner fixture source is missing.' }

# Match the real runner's locked interpreter precedence before creating fixtures.
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


function Test-RunnerCase([string]$Name, [string]$Suite, [string]$LuaSource, [string[]]$Required, [int]$ExpectedExit) {
    $temporaryParent = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\', '/')
    $scratch = Join-Path $temporaryParent ('live-galaxy-runner-' + [guid]::NewGuid().ToString('N'))
    try {
        $tests = Join-Path $scratch 'extensions/live_galaxy/tests'
        $modules = Join-Path $scratch 'extensions/live_galaxy/lua'
        $null = New-Item -ItemType Directory -Path $tests, $modules, (Join-Path $scratch 'tools')
        $copiedRunner = Join-Path $tests 'run_contracts.ps1'
        Copy-Item -LiteralPath $runner -Destination $copiedRunner
        $copiedLock = Join-Path $scratch 'tools/lua-runner.lock.json'
        Copy-Item -LiteralPath $lockPath -Destination $copiedLock
        foreach ($requiredPath in @($pwsh, $copiedRunner, $copiedLock)) {
            if (-not (Test-Path -LiteralPath $requiredPath -PathType Leaf)) { throw 'Runner fixture preparation failed.' }
        }
        [IO.File]::WriteAllText((Join-Path $modules 'runner_fixture.lua'), 'return "FIXTURE_TOKEN"')
        [IO.File]::WriteAllText((Join-Path $tests 'x4_discovery_contract.lua'), $LuaSource)
        if ($Suite -eq 'component_discovery') {
            [IO.File]::WriteAllText((Join-Path $tests 'component_discovery_binding_contract.ps1'),
                "Write-Output 'FIXTURE_BINDING_REACHED'; exit 23")
        }
        $output = @(& $pwsh -NoProfile -File $copiedRunner -Suite $Suite -LuaPath $lua 2>&1)
        $code = $LASTEXITCODE
        if (($ExpectedExit -eq 0 -and $code -ne 0) -or ($ExpectedExit -ne 0 -and $code -eq 0)) {
            throw "Runner fixture $Name returned unexpected exit $code. $($output -join ' ')"
        }
        $text = $output -join [Environment]::NewLine
        foreach ($marker in $Required) {
            if (-not $text.Contains($marker)) { throw "Runner fixture $Name did not reach $marker. $text" }
        }
        Write-Output "PASS runner fixture: $Name (exit=$code)"
    }
    finally {
        if (Test-Path -LiteralPath $scratch) {
            $resolved = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $scratch).Path)
            if ($resolved -cne $scratch -or (Split-Path -Parent $resolved) -cne $temporaryParent) {
                throw 'Unsafe runner fixture cleanup target.'
            }
            foreach ($item in @((Get-Item -LiteralPath $scratch)) + @(Get-ChildItem -LiteralPath $scratch -Recurse -Force)) {
                if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                    throw 'Refusing runner fixture cleanup through a reparse point.'
                }
            }
            Remove-Item -LiteralPath $resolved -Recurse -Force
        }
    }
}

$import = 'local token = require("live_galaxy/lua/runner_fixture"); assert(token == "FIXTURE_TOKEN"); '
Test-RunnerCase 'positive' 'x4_discovery' ($import + 'return { FIXTURE_READY = function() assert(token == "FIXTURE_TOKEN") end }') @('FIXTURE_READY', 'CASES x4_discovery_contract.lua: 1') 0
Test-RunnerCase 'lua-assertion' 'x4_discovery' ($import + 'return { fails = function() error("FIXTURE_LUA_ASSERTION") end }') @('FIXTURE_LUA_ASSERTION', 'Lua contract failed: x4_discovery_contract.lua') 1
Test-RunnerCase 'empty-cases' 'x4_discovery' ($import + 'print("FIXTURE_EMPTY_REACHED"); return {}') @('FIXTURE_EMPTY_REACHED', 'EMPTY_LUA_CASE_TABLE', 'Lua contract failed: x4_discovery_contract.lua') 1
Test-RunnerCase 'binding-failure' 'component_discovery' '' @('FIXTURE_BINDING_REACHED', 'Component discovery binding contract failed.') 1
