[CmdletBinding()]
param(
    [switch]$Verify,
    [string]$CompilerPath
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$lockPath = Join-Path $PSScriptRoot 'lua-runner.lock.json'

function Read-LuaLock {
    if (-not (Test-Path -LiteralPath $lockPath -PathType Leaf)) {
        throw "Lua runner lock is missing: $lockPath"
    }

    $lock = Get-Content -LiteralPath $lockPath -Raw | ConvertFrom-Json
    $required = @(
        $lock.compatibilityTarget,
        $lock.archiveVersion,
        $lock.executableVersion,
        $lock.platform,
        $lock.archive.url,
        $lock.archive.sha256,
        $lock.materialization.kind,
        $lock.materialization.compiler,
        $lock.materialization.executableRelativePath,
        $lock.materialization.linker
    )
    if ($required | Where-Object { [string]::IsNullOrWhiteSpace($_) }) {
        throw 'Lua runner lock has a required empty field.'
    }
    if ($lock.materialization.kind -ne 'source-build' -or
        $lock.materialization.compiler -ne 'clang.exe' -or
        $lock.materialization.linker -ne 'lld-link.exe') {
        throw 'Lua runner lock permits only the approved LLVM clang source-build route.'
    }
    if ($lock.archive.sha256 -notmatch '^[a-f0-9]{64}$') {
        throw 'Lua runner lock has an invalid archive SHA-256.'
    }
    if ($lock.platform -ne 'windows-x86_64') {
        throw "Lua runner lock platform is unsupported: $($lock.platform)"
    }
    return $lock
}

function Get-LockedExecutablePath([object]$Lock) {
    return Join-Path $root $Lock.materialization.executableRelativePath
}

function Get-LuaVersionOutput([string]$ExecutablePath) {
    $previousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        return (& $ExecutablePath -v 2>&1 | ForEach-Object {
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
}

function Assert-LockedExecutable([string]$ExecutablePath, [object]$Lock) {
    if (-not (Test-Path -LiteralPath $ExecutablePath -PathType Leaf)) {
        throw "Locked Lua executable is absent: $ExecutablePath. Run tools/provision-lua.ps1 with an approved GCC toolchain."
    }

    $reported = Get-LuaVersionOutput $ExecutablePath
    if ($LASTEXITCODE -ne 0 -or $reported -notmatch "(?m)^$([regex]::Escape($Lock.executableVersion))(?:\s|$)") {
        throw "Locked Lua executable version mismatch. Expected '$($Lock.executableVersion)'; got '$reported'."
    }
}

function Resolve-LockedTool([string]$Candidate, [string]$ExpectedName, [string]$Source) {
    if ([string]::IsNullOrWhiteSpace($Candidate)) {
        return $null
    }
    if (-not (Test-Path -LiteralPath $Candidate -PathType Leaf)) {
        throw "Configured $Source tool does not exist: $Candidate"
    }
    $resolved = (Resolve-Path -LiteralPath $Candidate).Path
    if ((Split-Path -Leaf $resolved) -ne $ExpectedName) {
        throw "Configured $Source tool must be ${ExpectedName}: $resolved"
    }
    return $resolved
}

$lock = Read-LuaLock
$executablePath = Get-LockedExecutablePath $lock

if ($Verify) {
    Assert-LockedExecutable $executablePath $lock
    Write-Output "Verified $($lock.executableVersion): $executablePath"
    exit 0
}

$compiler = Resolve-LockedTool $CompilerPath $lock.materialization.compiler '-CompilerPath'
if ($null -eq $compiler) {
    $compiler = Resolve-LockedTool $env:LIVE_GALAXY_C_COMPILER $lock.materialization.compiler 'LIVE_GALAXY_C_COMPILER'
}
if ($null -eq $compiler) {
    $compilerCommand = Get-Command $lock.materialization.compiler -ErrorAction SilentlyContinue
    if ($null -ne $compilerCommand) {
        $compiler = Resolve-LockedTool $compilerCommand.Source $lock.materialization.compiler 'PATH'
    }
}
if ($null -eq $compiler) {
    throw "Required approved C toolchain '$($lock.materialization.compiler)' is unavailable. No download or global installation was attempted."
}
$linker = Resolve-LockedTool (Join-Path (Split-Path -Parent $compiler) $lock.materialization.linker) $lock.materialization.linker 'compiler directory'
if ($null -eq $linker) {
    $linkerCommand = Get-Command $lock.materialization.linker -ErrorAction SilentlyContinue
    if ($null -ne $linkerCommand) {
        $linker = Resolve-LockedTool $linkerCommand.Source $lock.materialization.linker 'PATH'
    }
}
if ($null -eq $linker) {
    throw "Required approved linker '$($lock.materialization.linker)' is unavailable. No download or global installation was attempted."
}
$tar = Get-Command tar.exe -ErrorAction SilentlyContinue
if ($null -eq $tar) {
    throw 'Required archive extractor tar.exe is unavailable. No download or global installation was attempted.'
}

$cacheRoot = Join-Path $root 'tools/.cache/lua'
$versionRoot = Join-Path $cacheRoot $lock.archiveVersion
$archivePath = Join-Path $versionRoot "lua-$($lock.archiveVersion).tar.gz"
$sourceRoot = Join-Path $versionRoot "lua-$($lock.archiveVersion)"
$sourceDirectory = Join-Path $sourceRoot 'src'

New-Item -ItemType Directory -Force -Path $versionRoot | Out-Null
if (-not (Test-Path -LiteralPath $archivePath -PathType Leaf)) {
    Invoke-WebRequest -Uri $lock.archive.url -OutFile $archivePath
}

$actualSha256 = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actualSha256 -ne $lock.archive.sha256) {
    throw "Lua source archive integrity check failed. Expected $($lock.archive.sha256); got $actualSha256."
}

if (Test-Path -LiteralPath $sourceRoot) {
    Remove-Item -LiteralPath $sourceRoot -Recurse -Force
}
& $tar.Source -xf $archivePath -C $versionRoot
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $sourceDirectory -PathType Container)) {
    throw "Unable to extract verified Lua source archive: $archivePath"
}

$sources = Get-ChildItem -LiteralPath $sourceDirectory -Filter '*.c' |
    Where-Object { $_.Name -ne 'luac.c' } |
    Select-Object -ExpandProperty FullName
if ($sources.Count -eq 0) {
    throw "Verified Lua source archive did not contain C sources: $sourceDirectory"
}

$executableDirectory = Split-Path -Parent $executablePath
New-Item -ItemType Directory -Force -Path $executableDirectory | Out-Null
& $compiler -O2 -std=c99 -DLUA_USE_WINDOWS -fuse-ld=lld -o $executablePath @sources
if ($LASTEXITCODE -ne 0) {
    throw "Lua source build failed using $compiler and $linker."
}

Assert-LockedExecutable $executablePath $lock
Write-Output "Provisioned $($lock.executableVersion): $executablePath"
