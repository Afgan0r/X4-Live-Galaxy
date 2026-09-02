[CmdletBinding()]
param(
    [switch]$Verify,
    [string]$CompilerPath,
    [switch]$WithBusted,
    [string]$BustedTree = 'tools/.cache/luarocks-tree'
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

function Initialize-Busted([object]$Lock, [string]$OriginalExecutable) {
    Assert-LockedExecutable $OriginalExecutable $Lock
    $originalHash = (Get-FileHash -LiteralPath $OriginalExecutable -Algorithm SHA256).Hash
    $developmentRoot = Join-Path $root $Lock.bustedDevelopment.rootRelativePath
    $lua = Join-Path $developmentRoot 'bin/lua.exe'
    $tree = [IO.Path]::GetFullPath((Join-Path $root $BustedTree))
    $cache = [IO.Path]::GetFullPath((Join-Path $root 'tools/.cache'))
    if (-not $tree.StartsWith($cache + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Busted dependency tree must be inside tools/.cache.'
    }
    $launcher = Join-Path $tree 'bin/busted.bat'
    if ($Verify) {
        Assert-LockedExecutable $lua $Lock
        if (-not (Test-Path -LiteralPath $launcher -PathType Leaf)) {
            throw 'Busted is absent. Run tools/provision-lua.ps1 -WithBusted first.'
        }
        & $launcher --version
        if ($LASTEXITCODE -ne 0) { throw 'Installed Busted could not start.' }
        return
    }

    $savedEnvironment = @{}
    foreach ($name in @('PATH', 'INCLUDE', 'LIB', 'TEMP', 'TMP', 'LUAROCKS_CONFIG', 'LUA_PATH', 'LUA_CPATH')) {
        $savedEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
    }
    Push-Location $PSScriptRoot
    try {
        $compiler = Resolve-LockedTool $CompilerPath 'clang.exe' '-CompilerPath'
        if (-not $compiler) {
            $compiler = Resolve-LockedTool $env:LIVE_GALAXY_C_COMPILER 'clang.exe' 'LIVE_GALAXY_C_COMPILER'
        }
        if (-not $compiler) {
            $command = Get-Command clang.exe -ErrorAction SilentlyContinue
            if ($command) { $compiler = $command.Source }
        }
        if (-not $compiler) { throw 'Specify the installed clang.exe with -CompilerPath.' }
        $llvmRoot = Split-Path -Parent $compiler
        $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
        if (-not (Test-Path -LiteralPath $vswhere)) { throw 'MSVC Build Tools are required; no global installation was attempted.' }
        $vsRoot = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if (-not $vsRoot) { throw 'An installed x64 MSVC toolchain is required.' }
        $vcRoot = Get-ChildItem -LiteralPath (Join-Path $vsRoot 'VC/Tools/MSVC') -Directory |
            Sort-Object Name -Descending | Select-Object -First 1 -ExpandProperty FullName
        $sdkRoot = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits/10'
        $sdkVersion = Get-ChildItem -LiteralPath (Join-Path $sdkRoot 'Include') -Directory |
            Sort-Object Name -Descending | Select-Object -First 1 -ExpandProperty Name
        foreach ($required in @("$vcRoot/bin/Hostx64/x64/cl.exe", "$vcRoot/lib/x64/vcruntime.lib",
            "$sdkRoot/Include/$sdkVersion/ucrt/stdio.h", "$sdkRoot/Lib/$sdkVersion/um/x64/kernel32.lib",
            "$llvmRoot/lld-link.exe", "$llvmRoot/llvm-readobj.exe")) {
            if (-not (Test-Path -LiteralPath $required -PathType Leaf)) { throw "Missing native build prerequisite: $required" }
        }
        $env:PATH = "$vcRoot/bin/Hostx64/x64;$sdkRoot/bin/$sdkVersion/x64;$llvmRoot;" + $env:PATH
        $env:INCLUDE = "$vcRoot/include;$sdkRoot/Include/$sdkVersion/ucrt;$sdkRoot/Include/$sdkVersion/um;$sdkRoot/Include/$sdkVersion/shared"
        $env:LIB = "$vcRoot/lib/x64;$sdkRoot/Lib/$sdkVersion/ucrt/x64;$sdkRoot/Lib/$sdkVersion/um/x64"
        # LuaRocks 3.13.0 requires an absolute TEMP without spaces for its child commands.
        $env:TEMP = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'Temp/live-galaxy-260903-32k'
        if ($env:TEMP.Contains(' ')) { throw 'LuaRocks setup requires a task-specific Windows TEMP path without spaces.' }
        $env:TMP = $env:TEMP
        New-Item -ItemType Directory -Force -Path $env:TEMP, $developmentRoot,
            "$developmentRoot/bin", "$developmentRoot/lib", "$developmentRoot/include" | Out-Null
        $archive = Join-Path $developmentRoot "lua-$($Lock.archiveVersion).tar.gz"
        if (-not (Test-Path -LiteralPath $archive)) {
            $existingArchive = Join-Path $root "tools/.cache/lua/$($Lock.archiveVersion)/lua-$($Lock.archiveVersion).tar.gz"
            if (Test-Path -LiteralPath $existingArchive) { Copy-Item -LiteralPath $existingArchive -Destination $archive }
            else { Invoke-WebRequest -Uri $Lock.archive.url -OutFile $archive }
        }
        if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $Lock.archive.sha256) {
            throw 'Lua source archive checksum mismatch.'
        }
        $source = Join-Path $developmentRoot "lua-$($Lock.archiveVersion)/src"
        if (-not (Test-Path -LiteralPath $source)) {
            & tar.exe -xf $archive -C $developmentRoot
            if ($LASTEXITCODE -ne 0) { throw 'Lua source extraction failed.' }
        }
        $sources = Get-ChildItem -LiteralPath $source -Filter '*.c' |
            Where-Object Name -notin @('lua.c', 'luac.c', 'print.c') | Select-Object -ExpandProperty FullName
        & $compiler -O2 -std=c99 -DLUA_USE_WINDOWS -DLUA_BUILD_AS_DLL -fms-runtime-lib=dll -shared -fuse-ld=lld `
            "-Wl,/IMPLIB:$developmentRoot/lib/lua51.lib" -o "$developmentRoot/bin/lua51.dll" @sources
        if ($LASTEXITCODE -ne 0) { throw 'Lua development DLL build failed.' }
        & $compiler -O2 -std=c99 -DLUA_USE_WINDOWS -DLUA_BUILD_AS_DLL -fms-runtime-lib=dll -fuse-ld=lld `
            -o $lua "$source/lua.c" "$developmentRoot/lib/lua51.lib"
        if ($LASTEXITCODE -ne 0) { throw 'Lua development executable build failed.' }
        Copy-Item -LiteralPath "$source/lua.h", "$source/luaconf.h", "$source/lauxlib.h", "$source/lualib.h" -Destination "$developmentRoot/include"
        Assert-LockedExecutable $lua $Lock
        $exports = & "$llvmRoot/llvm-readobj.exe" --coff-exports "$developmentRoot/bin/lua51.dll"
        if ($LASTEXITCODE -ne 0 -or ($exports -join "`n") -notmatch 'Name: lua_gettop') { throw 'Lua DLL exports are absent.' }

        $rocksRoot = Join-Path $cache "luarocks/$($Lock.bustedDevelopment.luarocks.version)"
        New-Item -ItemType Directory -Force -Path $rocksRoot | Out-Null
        $rocksArchive = Join-Path $rocksRoot 'luarocks.zip'
        if (-not (Test-Path -LiteralPath $rocksArchive)) {
            Invoke-WebRequest -Uri $Lock.bustedDevelopment.luarocks.url -OutFile $rocksArchive
        }
        if ((Get-FileHash -LiteralPath $rocksArchive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $Lock.bustedDevelopment.luarocks.sha256) {
            throw 'LuaRocks archive checksum mismatch.'
        }
        $rocksExe = Join-Path $rocksRoot "luarocks-$($Lock.bustedDevelopment.luarocks.version)-windows-64/luarocks.exe"
        if (-not (Test-Path -LiteralPath $rocksExe)) { Expand-Archive -LiteralPath $rocksArchive -DestinationPath $rocksRoot }
        $config = Join-Path $rocksRoot 'config.lua'
        $cacheLuaPath = (Join-Path $cache 'luarocks-downloads').Replace('\', '/')
        @(
            'lua_version = "5.1"',
            'platforms = { "windows", "win32" }',
            'rocks_servers = { "https://luarocks.org" }',
            "local_cache = [[$cacheLuaPath]]",
            'check_certificates = true',
            'connection_timeout = 15',
            'variables = { CC = "cl", LD = "link", CFLAGS = "/nologo /MD /O2" }'
        ) | Set-Content -LiteralPath $config -Encoding utf8
        $env:LUAROCKS_CONFIG = $config
        # Exact standard rockspec constraints replace --pin's broken virtual-Lua replay.
        & $rocksExe --lua-version=5.1 "--lua-dir=$developmentRoot" "--tree=$tree" make live-galaxy-tests-1.0-1.rockspec --only-deps
        if ($LASTEXITCODE -ne 0) { throw 'Busted dependency installation failed; existing runner is preserved.' }
        $env:LUA_PATH = "$tree/share/lua/5.1/?.lua;$tree/share/lua/5.1/?/init.lua"
        $env:LUA_CPATH = "$tree/lib/lua/5.1/?.dll"
        & $lua -e 'assert(require("system")); assert(require("term.core")); assert(require("lfs")); assert(require("busted.runner")); print("Native Busted dependencies loaded")'
        if ($LASTEXITCODE -ne 0) { throw 'Native Busted dependency loading failed.' }
        & $launcher --version
        if ($LASTEXITCODE -ne 0) { throw 'Busted launcher failed.' }
        Write-Output "Provisioned full Busted in $tree"
    }
    finally {
        Pop-Location
        foreach ($name in $savedEnvironment.Keys) {
            [Environment]::SetEnvironmentVariable($name, $savedEnvironment[$name], 'Process')
        }
        if ((Get-FileHash -LiteralPath $OriginalExecutable -Algorithm SHA256).Hash -ne $originalHash) {
            throw 'Original Lua executable changed during Busted setup.'
        }
        Assert-LockedExecutable $OriginalExecutable $Lock
        Write-Output "Original Lua preserved: SHA256 $originalHash"
    }
}

$lock = Read-LuaLock
$executablePath = Get-LockedExecutablePath $lock

if ($WithBusted) {
    Initialize-Busted $lock $executablePath
    exit 0
}

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
