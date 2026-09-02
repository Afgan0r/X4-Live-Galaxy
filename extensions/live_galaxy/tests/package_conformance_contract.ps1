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

foreach ($required in @('lua/live_galaxy_x4_discovery.lua', 'lua/live_galaxy_telemetry.lua', 'lua/live_galaxy_normalize.lua')) {
    if ($output -notcontains "IMPORT $required") { throw "Missing transitive import: $required" }
}
if ($output -notcontains 'NATIVE lua/live_galaxy_component_discovery.lua') {
    throw 'Actual native acquisition was not followed.'
}

function Set-FixtureFile([string]$Root, [string]$RelativePath, [string]$Content) {
    $path = Join-Path $Root $RelativePath
    $null = New-Item -ItemType Directory -Path (Split-Path -Parent $path) -Force
    [IO.File]::WriteAllText($path, $Content)
}

function Test-PackageCase([string]$Name, [string]$Expected, [scriptblock]$Edit) {
    $temporaryParent = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\', '/')
    $scratch = Join-Path $temporaryParent ('live-galaxy-package-' + [guid]::NewGuid().ToString('N'))
    $fixture = Join-Path $scratch 'package'
    try {
        $null = New-Item -ItemType Directory -Path $fixture
        foreach ($path in @('content.xml', 'ui.xml', 'lua')) {
            Copy-Item -LiteralPath (Join-Path $packageRoot $path) -Destination $fixture -Recurse
        }
        & $Edit $fixture
        $result = @(& pwsh -NoProfile -File $checker -PackageRoot $fixture 2>&1)
        $code = $LASTEXITCODE
        if ($Expected -eq 'PASS') {
            if ($code -ne 0 -or $result -notcontains 'PASS package: live_galaxy (local static evidence)') {
                throw "Fixture $Name failed: $($result -join ' ')"
            }
        }
        elseif ($code -ne 1 -or ($result -join "`n") -notmatch "(?m)^FAIL package: $([regex]::Escape($Expected))(?: |$)") {
            throw "Fixture $Name expected $Expected, exit=$code, output=$($result -join ' ')"
        }
        if (($result -join "`n").Contains($scratch)) { throw "Fixture $Name leaked an absolute path." }
        Write-Output "PASS package fixture: $Name ($Expected)"
    }
    finally {
        if (Test-Path -LiteralPath $scratch) {
            $resolved = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $scratch).Path)
            if ($resolved -cne $scratch -or (Split-Path -Parent $resolved) -cne $temporaryParent) {
                throw 'Unsafe package fixture cleanup target.'
            }
            # Fixtures never create links; refuse traversal if one appeared.
            foreach ($item in @((Get-Item -LiteralPath $scratch)) + @(Get-ChildItem -LiteralPath $scratch -Recurse -Force)) {
                if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                    throw 'Refusing package fixture cleanup through a reparse point.'
                }
            }
            Remove-Item -LiteralPath $resolved -Recurse -Force
        }
    }
}

$entry = 'lua/live_galaxy_runtime.lua'
$binding = 'local ffi = require("ffi"); local C = ffi.C'
Test-PackageCase 'literal-import' 'PASS' {
    param($p)
    Set-FixtureFile $p $entry ($binding + '; local dep = require("live_galaxy/lua/dependency")')
    Set-FixtureFile $p 'lua/dependency.lua' 'return {}'
}
Test-PackageCase 'static-concatenation-and-decoys' 'PASS' {
    param($p)
    Set-FixtureFile $p $entry @'
-- require("test/ignored") ffi.C
--[=[ require("test/ignored") local C = ffi.C ]=]
local decoy = [==[require("test/ignored") local C = ffi.C]==]
local quoted = "require"
local escaped = "\" require('test/ignored') local C = ffi.C"
local PREFIX = "live_galaxy/lua/"
local ffi = require("ffi")
local native = ffi.C
local dependency = require(PREFIX .. "dependency")
'@
    Set-FixtureFile $p 'lua/dependency.lua' 'return {}'
}
$helper = @'
local PREFIX = "live_galaxy/lua/"
local function load_module(name)
    return require(PREFIX .. name)
end
local ffi = require("ffi")
local C = ffi.C
'@
Test-PackageCase 'static-helper' 'PASS' {
    param($p)
    Set-FixtureFile $p $entry ($helper + "`nlocal dependency = load_module('dependency')")
    Set-FixtureFile $p 'lua/dependency.lua' 'return {}'
}
Test-PackageCase 'missing-manifest' 'MISSING_CONTENT_MANIFEST' { param($p) Remove-Item -LiteralPath (Join-Path $p 'content.xml') }
Test-PackageCase 'wrong-identity' 'INVALID_PACKAGE_IDENTITY' { param($p) Set-FixtureFile $p 'content.xml' '<content id="wrong"><dependency id="ws_2042901274" /></content>' }
Test-PackageCase 'missing-identity' 'INVALID_PACKAGE_IDENTITY' { param($p) Set-FixtureFile $p 'content.xml' '<content><dependency id="ws_2042901274" /></content>' }
Test-PackageCase 'missing-dependency' 'INVALID_PACKAGE_IDENTITY' { param($p) Set-FixtureFile $p 'content.xml' '<content id="live_galaxy" />' }
Test-PackageCase 'wrong-dependency' 'INVALID_PACKAGE_IDENTITY' { param($p) Set-FixtureFile $p 'content.xml' '<content id="live_galaxy"><dependency id="wrong" /></content>' }
Test-PackageCase 'invalid-xml' 'INVALID_XML' { param($p) Set-FixtureFile $p 'content.xml' '<content' }
Test-PackageCase 'missing-ui' 'MISSING_UI_REGISTRATION' { param($p) Remove-Item -LiteralPath (Join-Path $p 'ui.xml') }
foreach ($case in @(
    @('missing-environment', '<addon name="live_galaxy" />', 'MISSING_REGISTRATION'),
    @('wrong-environment', '<addon name="live_galaxy"><environment type="game" /></addon>', 'MISSING_REGISTRATION'),
    @('missing-registration', '<addon name="live_galaxy"><environment type="menus"><dependency name="sn_mod_support_apis" /></environment></addon>', 'MISSING_REGISTRATION'),
    @('wrong-ui-dependency', '<addon name="live_galaxy"><environment type="menus"><dependency name="wrong" /><file name="lua/live_galaxy_runtime.lua" /></environment></addon>', 'MISSING_REGISTRATION'),
    @('wrong-entrypoint', '<addon name="live_galaxy"><environment type="menus"><dependency name="sn_mod_support_apis" /><file name="lua/wrong.lua" /></environment></addon>', 'WRONG_ENTRYPOINT')
)) {
    Test-PackageCase $case[0] $case[2] { param($p) Set-FixtureFile $p 'ui.xml' $case[1] }
}
Test-PackageCase 'missing-entrypoint' 'UNRESOLVED_IMPORT' { param($p) Remove-Item -LiteralPath (Join-Path $p $entry) }
Test-PackageCase 'missing-transitive-module' 'UNRESOLVED_IMPORT' { param($p) Remove-Item -LiteralPath (Join-Path $p 'lua/live_galaxy_normalize.lua') }
foreach ($case in @(
    @('wrong-case', 'require("live_galaxy/lua/Live_Galaxy_Normalize")', 'UNRESOLVED_IMPORT'),
    @('bare-import', 'require("live_galaxy_normalize")', 'BARE_PRODUCTION_IMPORT'),
    @('test-only-import', 'require("tests/helper")', 'TEST_ONLY_DEPENDENCY'),
    @('dynamic-import', 'require(get_name())', 'DYNAMIC_REQUIRE'),
    @('root-escape', 'require("live_galaxy/lua/../outside")', 'ROOT_ESCAPE'),
    @('cycle', 'require("live_galaxy/lua/live_galaxy_runtime")', 'IMPORT_CYCLE'),
    @('alias', 'local loader = require', 'REQUIRE_ALIAS_UNSUPPORTED')
)) {
    Test-PackageCase $case[0] $case[2] { param($p) Set-FixtureFile $p $entry ($binding + '; ' + $case[1]) }
}
Test-PackageCase 'dynamic-helper' 'DYNAMIC_REQUIRE' { param($p) Set-FixtureFile $p $entry ($helper + "`nlocal dep = load_module(get_name())") }
Test-PackageCase 'helper-alias' 'REQUIRE_HELPER_ALIAS_UNSUPPORTED' { param($p) Set-FixtureFile $p $entry ($helper + "`nlocal loader = load_module") }
foreach ($decoy in @('-- local C = ffi.C', '--[=[ local C = ffi.C ]=]', 'local text = "local C = ffi.C"', "local text = 'local C = ffi.C'", 'local text = [==[ local C = ffi.C ]==]')) {
    Test-PackageCase "missing-native-$($decoy.Substring(0, 2))" 'NATIVE_BINDING_NOT_FOUND' {
        param($p) Set-FixtureFile $p $entry ("local ffi = require('ffi')`n" + $decoy)
    }
}
foreach ($access in @('local other = ffi.C', 'target = ffi.C', 'ffi.C.Call()', 'local other = ffi["C"]', 'local other = (ffi.C)')) {
    Test-PackageCase "duplicate-native-$access" 'ALTERNATE_BINDING_SOURCE' { param($p) Set-FixtureFile $p $entry ($binding + '; ' + $access) }
}
Test-PackageCase 'alternate-only-native' 'ALTERNATE_BINDING_SOURCE' { param($p) Set-FixtureFile $p $entry 'local C = globals.C' }
Test-PackageCase 'file-byte-bound' 'FILE_BYTES_EXCEEDED' { param($p) Set-FixtureFile $p $entry (' ' * 32769) }
Test-PackageCase 'depth-bound' 'GRAPH_DEPTH_EXCEEDED' {
    param($p)
    Set-FixtureFile $p $entry ($binding + '; require("live_galaxy/lua/chain1")')
    foreach ($i in 1..5) { Set-FixtureFile $p "lua/chain$i.lua" "require('live_galaxy/lua/chain$($i + 1)')" }
}
Test-PackageCase 'file-count-bound' 'GRAPH_SIZE_EXCEEDED' {
    param($p)
    $source = $binding
    foreach ($i in 1..8) {
        $source += "; require('live_galaxy/lua/leaf$i')"
        Set-FixtureFile $p "lua/leaf$i.lua" 'return {}'
    }
    Set-FixtureFile $p $entry $source
}
Test-PackageCase 'total-byte-bound' 'TOTAL_BYTES_EXCEEDED' {
    param($p)
    $source = $binding
    foreach ($i in 1..5) {
        $source += "; require('live_galaxy/lua/leaf$i')"
        Set-FixtureFile $p "lua/leaf$i.lua" (' ' * 30000)
    }
    Set-FixtureFile $p $entry $source
}
Test-PackageCase 'import-count-bound' 'IMPORT_COUNT_EXCEEDED' {
    param($p)
    $source = $binding
    foreach ($i in 1..8) {
        $source += "; require('live_galaxy/lua/leaf$i')"
        Set-FixtureFile $p "lua/leaf$i.lua" 'require("ffi"); require("extensions.sn_mod_support_apis.ui.named_pipes.Interface")'
    }
    Set-FixtureFile $p $entry $source
}
