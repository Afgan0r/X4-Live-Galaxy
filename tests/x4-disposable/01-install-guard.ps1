[CmdletBinding()]
param([switch]$VerifyPackageOnly)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$packageRelative = 'dist/live-galaxy-carrier-b'
$package = Join-Path $root $packageRelative
$extension = Join-Path $package 'extensions/live_galaxy'
. (Join-Path $root 'tools/carrier-b-package-contract.ps1')

if (Get-Process -Name 'X4' -ErrorAction SilentlyContinue) {
    throw 'refusing verification or installation while X4 is running'
}

$manifestPath = Join-Path $package 'manifest.json'
if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
    throw 'ready package manifest is missing'
}
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
Assert-CarrierBManifest $manifest 'ready-for-user-x4-checkpoint'
$packageInputs = @(
    'Cargo.toml', 'Cargo.lock', 'config/carrier-b-limits.json',
    'crates/x4-carrier-native', 'crates/x4-bridge', 'extensions/live_galaxy',
    'tools/package-live-galaxy.ps1', 'tools/carrier-b-package-contract.ps1'
)
git -C $root merge-base --is-ancestor $manifest.source_revision HEAD
if ($LASTEXITCODE -ne 0) { throw 'package source revision is not an ancestor of HEAD' }
git -C $root diff --quiet HEAD -- @packageInputs
if ($LASTEXITCODE -ne 0) { throw 'package inputs have uncommitted changes' }
$untrackedInputs = @(git -C $root ls-files --others --exclude-standard -- @packageInputs)
if ($untrackedInputs.Count -ne 0) { throw 'package inputs have untracked changes' }
git -C $root diff --quiet $manifest.source_revision HEAD -- @packageInputs
if ($LASTEXITCODE -ne 0) { throw 'package inputs changed after the packaged source revision' }
$requiredFiles = @(
    'carrier-b-limits.json', 'extensions/live_galaxy/ui_c_library_live_galaxy_carrier_64.txt',
    'live-galaxy-bridge.exe', 'STARTUP.txt'
)
$manifestFiles = @($manifest.files.PSObject.Properties.Name)
if (@($requiredFiles | Where-Object { $_ -cnotin $manifestFiles }).Count -ne 0) {
    throw 'package manifest omits a required Carrier B file'
}
Assert-CarrierBBundleFiles $package $manifest

$content = [xml](Get-Content -LiteralPath (Join-Path $extension 'content.xml') -Raw)
$ui = [xml](Get-Content -LiteralPath (Join-Path $extension 'ui.xml') -Raw)
$md = [xml](Get-Content -LiteralPath (Join-Path $extension 'md/live_galaxy_observation.xml') -Raw)
$runtime = Get-Content -LiteralPath (Join-Path $extension 'lua/live_galaxy_runtime.lua') -Raw
$carrier = Get-Content -LiteralPath (Join-Path $extension 'lua/live_galaxy_carrier.lua') -Raw

if (@($content.SelectNodes('/content/dependency')).Count -ne 0) { throw 'unexpected content dependency' }
if (@($ui.SelectNodes('/addon/environment/dependency')).Count -ne 0) { throw 'unexpected UI dependency' }
if ($ui.addon.environment.type -ne 'menus' -or
    $ui.addon.environment.file.name -ne 'lua/live_galaxy_runtime.lua') {
    throw 'menus UI Lua registration is invalid'
}
$cues = @($md.SelectNodes('//cue[@checkinterval]'))
if ($cues.Count -ne 1 -or $cues[0].Name -ne 'live_galaxy_observation' -or $cues[0].checkinterval -ne '50ms') {
    throw 'single 50 ms Carrier B telemetry pump cadence is missing'
}
if ($runtime -notmatch 'RegisterEvent\("live_galaxy_observation", dispatch\)' -or
    $runtime -match '(?i)sn_mod_support_apis') {
    throw 'dependency-free direct event registration is invalid'
}
if ($carrier -notmatch [regex]::Escape('.\\extensions\\live_galaxy\\ui_c_library_live_galaxy_carrier_64.txt') -or
    $carrier -notmatch 'luaopen_live_galaxy_carrier') {
    throw 'owned native loader path or initializer is invalid'
}

Write-Output 'Carrier B install guard passed read-only: dependency-free package, 50 ms admitted pump, owned DLL and bridge.'
if ($VerifyPackageOnly) { exit 0 }
throw 'installation remains an explicit caller-owned copy after this guard'
