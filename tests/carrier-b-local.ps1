[CmdletBinding()]
param(
    [switch]$SelfTest,
    [ValidateSet('actual-chain', 'pending-io-unload')]
    [string]$Scenario = 'actual-chain',
    [ValidateSet('bridge-first', 'native-first')]
    [string]$StartupOrder = 'bridge-first',
    [string]$LuaJitPath,
    [string]$LuaLibraryPath,
    [string]$LimitsFile
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not $SelfTest) { throw 'SELF_TEST_REQUIRED' }
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$cache = Join-Path $repo 'tools/.cache/carrier-b-local'
$luaRoot = Join-Path $repo 'tools/.cache/lua-busted/5.1.5'
$luaSource = Join-Path $luaRoot 'lua-5.1.5/src'
$defaultLua = Join-Path $luaRoot 'bin/lua.exe'
$defaultLibrary = Join-Path $luaRoot 'bin/lua51.dll'
$LuaJitPath = if ($LuaJitPath) { (Resolve-Path $LuaJitPath).Path } else { $defaultLua }
$LuaLibraryPath = if ($LuaLibraryPath) { (Resolve-Path $LuaLibraryPath).Path } else { $defaultLibrary }
foreach ($path in @($LuaJitPath, $LuaLibraryPath, (Join-Path $luaSource 'lua.h'))) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "COMPATIBLE_LUA_TOOL_MISSING:$path" }
}
$reported = (& $LuaJitPath -v 2>&1 | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $reported -notmatch '^Lua 5\.1\.5') { throw 'COMPATIBLE_LUA_ABI_MISMATCH' }
if ($Scenario -eq 'actual-chain' -and $StartupOrder -eq 'bridge-first') {
    cargo test --locked -p observation-ingest --test wire_v2_contract
    if ($LASTEXITCODE -ne 0) { throw 'WIRE_VERSION_MATRIX_FAILED' }
    cargo test --locked -p x4-bridge --test carrier_b_contract --test carrier_b_recovery --test carrier_b_publication
    if ($LASTEXITCODE -ne 0) { throw 'BRIDGE_RECOVERY_MATRIX_FAILED' }
}

function Write-CalibrationLimits([string]$Path) {
    $json = '{"complete_message_bytes":2048,"control_message_bytes":512,"max_candidate_raw_bytes":2048,"max_candidate_records":1,"max_candidate_batches":1,"max_candidate_work":2048,"max_message_age_millis":5000,"max_message_inactivity_millis":5000,"max_candidates":2,"max_aggregate_bytes":4096,"max_aggregate_records":2,"max_aggregate_batches":2,"max_aggregate_work":4096,"max_publication_records":1,"max_publication_content_bytes":2048,"max_pending_bytes":4096,"max_total_bytes":8192,"max_lifecycle_work":8192,"max_delivery_attempts":2,"max_blockers":4,"reconnect_attempts":2,"reconnect_delay_millis":25,"availability_interval_millis":5000}'
    [IO.File]::WriteAllText($Path, $json, [Text.UTF8Encoding]::new($false))
}

function Initialize-CompilerEnvironment {
    $clang = 'C:\Program Files\LLVM\bin\clang.exe'
    if (-not (Test-Path -LiteralPath $clang)) { throw 'PINNED_CLANG_MISSING' }
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    $vsRoot = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (-not $vsRoot) { throw 'MSVC_BUILD_TOOLS_MISSING' }
    $vcRoot = Get-ChildItem -LiteralPath (Join-Path $vsRoot 'VC/Tools/MSVC') -Directory |
        Sort-Object Name -Descending | Select-Object -First 1 -ExpandProperty FullName
    $sdkRoot = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits/10'
    $sdkVersion = Get-ChildItem -LiteralPath (Join-Path $sdkRoot 'Include') -Directory |
        Sort-Object Name -Descending | Select-Object -First 1 -ExpandProperty Name
    $env:PATH = "$vcRoot/bin/Hostx64/x64;$sdkRoot/bin/$sdkVersion/x64;$(Split-Path -Parent $clang);$env:PATH"
    $env:INCLUDE = "$vcRoot/include;$sdkRoot/Include/$sdkVersion/ucrt;$sdkRoot/Include/$sdkVersion/um;$sdkRoot/Include/$sdkVersion/shared"
    $env:LIB = "$vcRoot/lib/x64;$sdkRoot/Lib/$sdkVersion/ucrt/x64;$sdkRoot/Lib/$sdkVersion/um/x64"
    $clang
}

function Build-Host([string]$Root) {
    $sourcePath = Join-Path $Root 'carrier-b-host.c'
    $hostPath = Join-Path $Root 'carrier-b-host.exe'
    $source = @'
#include <windows.h>
#include <stdio.h>
#include <string.h>
#include "lua.h"
#include "lauxlib.h"
#include "lualib.h"

static int host_sleep(lua_State *state) {
    Sleep((DWORD)luaL_checkinteger(state, 1));
    return 0;
}

static int run_state(int argc, char **argv, const char *mode, const char *prior) {
    lua_State *state = luaL_newstate();
    if (!state) return 10;
    luaL_openlibs(state);
    lua_pushcfunction(state, host_sleep);
    lua_setglobal(state, "host_sleep");
    if (luaL_loadfile(state, argv[2]) != 0) {
        fprintf(stderr, "load:%s\n", lua_tostring(state, -1)); lua_close(state); return 11;
    }
    lua_pushstring(state, argv[1]); lua_pushstring(state, mode);
    lua_pushstring(state, argv[3]); lua_pushstring(state, argv[4]); lua_pushstring(state, prior);
    if (lua_pcall(state, 5, 0, 0) != 0) {
        fprintf(stderr, "run:%s\n", lua_tostring(state, -1)); lua_close(state); return 12;
    }
    lua_close(state);
    return 0;
}

static int await_unload(void) {
    for (int i = 0; i < 400; ++i) {
        if (!GetModuleHandleW(L"ui_c_library_live_galaxy_carrier_64.txt")) return 0;
        Sleep(5);
    }
    return 21;
}

int main(int argc, char **argv) {
    if (argc != 6) return 9;
    int code = run_state(argc, argv, argv[5], "");
    if (code || strcmp(argv[5], "pending-io-unload") != 0) return code;
    code = await_unload();
    if (code) { fprintf(stderr, "native image remained loaded\n"); return code; }
    FILE *result = fopen(argv[3], "rb"); char token[64] = {0};
    if (!result) return 22;
    char body[512] = {0}; fread(body, 1, sizeof(body) - 1, result); fclose(result);
    char *begin = strstr(body, "token=\"");
    if (!begin || sscanf(begin + 7, "%63[^\"]", token) != 1) return 23;
    code = run_state(argc, argv, "reload", token);
    if (code) return code;
    code = await_unload();
    if (code) return code;
    puts("PENDING_IO_TERMINAL_AND_MODULE_UNLOADED");
    return 0;
}
'@
    [IO.File]::WriteAllText($sourcePath, $source, [Text.UTF8Encoding]::new($false))
    $clang = Initialize-CompilerEnvironment
    $sources = Get-ChildItem -LiteralPath $luaSource -Filter '*.c' |
        Where-Object Name -notin @('lua.c', 'luac.c', 'print.c') | Select-Object -ExpandProperty FullName
    $exports = @('lua_createtable', 'lua_getmetatable', 'lua_gettop', 'lua_next', 'lua_pushcclosure',
        'lua_pushinteger', 'lua_pushlstring', 'lua_pushnil', 'lua_rawget', 'lua_setfield', 'lua_settop',
        'lua_toboolean', 'lua_tointeger', 'lua_tolstring', 'lua_tonumber', 'lua_type') |
        ForEach-Object { "-Wl,/EXPORT:$_" }
    & $clang -w -O2 -std=c99 -DLUA_USE_WINDOWS -fms-runtime-lib=dll -fuse-ld=lld "-I$luaSource" @exports -o $hostPath $sourcePath @sources
    if ($LASTEXITCODE -ne 0) { throw 'LUA_HOST_BUILD_FAILED' }
    $hostPath
}

function Stop-Owned([Diagnostics.Process]$Process) {
    if ($null -ne $Process -and -not $Process.HasExited) { $Process.Kill(); $Process.WaitForExit(2000) | Out-Null }
}

function Start-Owned([string]$Path, [string[]]$Arguments, [string]$WorkingDirectory, [bool]$Redirect = $false) {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $Path
    $info.WorkingDirectory = $WorkingDirectory
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $Redirect
    $info.RedirectStandardError = $Redirect
    foreach ($argument in $Arguments) { $info.ArgumentList.Add($argument) }
    $process = [Diagnostics.Process]::new(); $process.StartInfo = $info
    if (-not $process.Start()) { throw "PROCESS_START_FAILED:$Path" }
    $process
}

[IO.Directory]::CreateDirectory($cache) | Out-Null
$run = Join-Path $cache ([guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory((Join-Path $run 'extensions/live_galaxy/lua')) | Out-Null
Get-ChildItem -LiteralPath (Join-Path $repo 'extensions/live_galaxy/lua') -File |
    ForEach-Object { Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $run 'extensions/live_galaxy/lua') }
cargo build --locked --release -p x4-carrier-native -p x4-bridge
if ($LASTEXITCODE -ne 0) { throw 'RELEASE_BUILD_FAILED' }
Copy-Item -LiteralPath (Join-Path $repo 'target/release/x4_carrier_native.dll') -Destination (Join-Path $run 'extensions/live_galaxy/ui_c_library_live_galaxy_carrier_64.txt')
$limits = Join-Path $run 'limits.json'
if ($LimitsFile) { Copy-Item -LiteralPath (Resolve-Path $LimitsFile) -Destination $limits } else { Write-CalibrationLimits $limits }
$hostExecutable = Build-Host $run
$data = Join-Path $run 'data'; [IO.Directory]::CreateDirectory($data) | Out-Null
$result = Join-Path $run 'result.lua'; $marker = Join-Path $run 'kill.marker'
$bridge = $null
$timer = [Diagnostics.Stopwatch]::StartNew()
try {
    if ($StartupOrder -eq 'bridge-first') {
        $bridge = Start-Owned (Join-Path $repo 'target/release/x4-bridge.exe') @('--data-dir', $data, '--limits-file', $limits) $run
        $process = Start-Owned $hostExecutable @($run, (Join-Path $repo 'extensions/live_galaxy/tests/carrier_b_local.lua'), $result, $marker, $Scenario) $run $true
    } else {
        $process = Start-Owned $hostExecutable @($run, (Join-Path $repo 'extensions/live_galaxy/tests/carrier_b_local.lua'), $result, $marker, $Scenario) $run $true
        Start-Sleep -Milliseconds 25
        $bridge = Start-Owned (Join-Path $repo 'target/release/x4-bridge.exe') @('--data-dir', $data, '--limits-file', $limits) $run
    }
    if ($Scenario -eq 'pending-io-unload') {
        $deadline = [DateTime]::UtcNow.AddSeconds(10)
        while (-not (Test-Path -LiteralPath $marker) -and -not $process.HasExited -and [DateTime]::UtcNow -lt $deadline) { Start-Sleep -Milliseconds 10 }
        if (-not (Test-Path -LiteralPath $marker)) { throw 'PENDING_OPERATION_MARKER_MISSING' }
        Stop-Owned $bridge
    }
    if (-not $process.WaitForExit(30000)) { Stop-Owned $process; throw 'LUA_HOST_WATCHDOG' }
    if ($process.ExitCode -ne 0) { throw "LUA_HOST_FAILED:$($process.ExitCode):$($process.StandardError.ReadToEnd())" }
    $resultText = Get-Content -LiteralPath $result -Raw
    $luaResult = [pscustomobject]@{
        actual_native = $resultText -match 'actual_native=true'
        getter_calls = if ($resultText -match 'getter_calls=(\d+)') { [int]$Matches[1] } else { -1 }
        stale = if ($resultText -match 'stale=(-?\d+)') { [int]$Matches[1] } else { 0 }
        fresh = if ($resultText -match 'fresh="([^"]+)"') { $Matches[1] } else { '' }
        token = if ($resultText -match 'token="([^"]+)"') { $Matches[1] } else { '' }
    }
    if ($Scenario -eq 'pending-io-unload') {
        if ($luaResult.stale -ne -16 -or $luaResult.fresh -eq $luaResult.token) { throw 'STALE_GENERATION_FENCE_FAILED' }
    } else {
        $readback = & (Join-Path $repo 'target/release/x4-bridge.exe') --readback --data-dir $data --section-key carrier_b_realtime_sample --section-revision 1
        if ($LASTEXITCODE -ne 0 -or ($readback -join '') -notmatch '"section_revision":1') { throw 'DURABLE_READBACK_FAILED' }
        if (-not $luaResult.actual_native -or $luaResult.getter_calls -ne 1) { throw 'ACTUAL_LUA_NATIVE_IDENTITY_FAILED' }
    }
    $timer.Stop()
    $nativeHash = (Get-FileHash (Join-Path $run 'extensions/live_galaxy/ui_c_library_live_galaxy_carrier_64.txt') -Algorithm SHA256).Hash.ToLowerInvariant()
    $configHash = (Get-FileHash $limits -Algorithm SHA256).Hash.ToLowerInvariant()
    Write-Output "MEASUREMENT scenario=$Scenario startup_order=$StartupOrder elapsed_millis=$($timer.ElapsedMilliseconds) getter_calls=$($luaResult.getter_calls) native_sha256=$nativeHash config_sha256=$configHash"
    Write-Output "PASS scenario=$Scenario startup_order=$StartupOrder actual_native=true durable=$($Scenario -ne 'pending-io-unload')"
} finally {
    Stop-Owned $bridge
}
