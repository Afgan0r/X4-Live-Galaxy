param([switch]$VerifyPackageOnly)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$extension = Join-Path $root 'extensions/live_galaxy'
$content = [xml](Get-Content -Raw (Join-Path $extension 'content.xml'))
$ui = [xml](Get-Content -Raw (Join-Path $extension 'ui.xml'))
$md = [xml](Get-Content -Raw (Join-Path $extension 'md/live_galaxy_observation.xml'))

if ($content.content.id -ne 'live_galaxy' -or $content.content.dependency.id -ne 'sn_mod_support_apis') { throw 'extension dependency identity is invalid' }
if ($ui.addon.environment.type -ne 'menus' -or $ui.addon.environment.file.name -ne 'lua/live_galaxy_runtime.lua') { throw 'menus UI Lua registration is missing' }
if (-not (Test-Path (Join-Path $extension $ui.addon.environment.file.name))) { throw 'registered UI Lua file is missing' }
if ($md.mdscript.cues.cue.conditions.event_game_started -eq $null -or $md.mdscript.cues.cue.cues.cue.checkinterval -ne '30s') { throw 'MD scheduler activation or cadence is missing' }
if ($md.mdscript.cues.cue.cues.cue.actions.raise_lua_event.name -ne "'live_galaxy_observation'") { throw 'quoted MD event link is missing' }
if ($md.mdscript.cues.cue.cues.cue.actions.raise_lua_event.param -ne "'telemetry_tick'") { throw 'MD event payload is missing' }
if (-not (Test-Path (Join-Path $root 'target/debug/x4-bridge.exe'))) { throw 'bridge binary is missing' }
if ($VerifyPackageOnly) { exit 0 }
if (Get-Process -Name 'X4' -ErrorAction SilentlyContinue) { throw 'refusing installation while X4 is running' }
throw 'installation is intentionally deferred; use -VerifyPackageOnly'
