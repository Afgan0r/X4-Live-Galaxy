param([switch]$VerifyPackageOnly)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$extension = Join-Path $root 'extensions/live_galaxy'
$content = [xml](Get-Content -Raw (Join-Path $extension 'content.xml'))
$ui = [xml](Get-Content -Raw (Join-Path $extension 'ui.xml'))
$md = [xml](Get-Content -Raw (Join-Path $extension 'md/live_galaxy_observation.xml'))
$runtime = Get-Content -Raw (Join-Path $extension 'lua/live_galaxy_runtime.lua')
$discovery = Join-Path $extension 'lua/live_galaxy_x4_discovery.lua'
$traceConfig = Join-Path $extension 'lua/live_galaxy_trace_config.lua'

if ($content.content.id -ne 'live_galaxy' -or $content.content.dependency.id -ne 'ws_2042901274') { throw 'extension dependency identity is invalid' }
if ($ui.addon.environment.type -ne 'menus' -or $ui.addon.environment.file.name -ne 'lua/live_galaxy_runtime.lua' -or $ui.addon.environment.dependency.name -notcontains 'sn_mod_support_apis') { throw 'menus UI Lua registration or dependency is missing' }
if (-not (Test-Path (Join-Path $extension $ui.addon.environment.file.name))) { throw 'registered UI Lua file is missing' }
if (-not (Test-Path $discovery)) { throw 'runtime discovery adapter is missing' }
if ($md.mdscript.cues.cue.conditions.check_any.event_cue_signalled.cue -ne 'md.Setup.GameStart' -or $md.mdscript.cues.cue.conditions.check_any.event_game_loaded -eq $null -or $md.mdscript.cues.cue.cues.cue.instantiate -ne 'true' -or $md.mdscript.cues.cue.cues.cue.checkinterval -ne '30s') { throw 'MD scheduler activation or cadence is missing' }
if ($md.mdscript.cues.cue.cues.cue.actions.raise_lua_event.name -ne "'live_galaxy_observation'") { throw 'quoted MD event link is missing' }
if ($md.mdscript.cues.cue.cues.cue.actions.raise_lua_event.param -ne "'telemetry_tick'") { throw 'MD event payload is missing' }
if (-not (Test-Path $traceConfig)) { throw 'developer trace config is missing' }
$traceConfigSource = Get-Content -Raw $traceConfig
if ($traceConfigSource -notmatch 'version_diagnostic_enabled\s*=\s*false') { throw 'embedded version diagnostic must default to disabled' }
if ($traceConfigSource -notmatch 'capability_probe_enabled\s*=\s*false' -or $traceConfigSource -notmatch 'capability_probe_attempt_id\s*=\s*"unset"') { throw 'capability vector diagnostic must default to disabled with an inert attempt id' }
if ($runtime -notmatch 'VERSION_DIAGNOSTIC_MAX_BYTES = 64' -or $runtime -notmatch 'function runtime\.sanitize_embedded_version\(value\)' -or $runtime -notmatch 'type\(value\) ~= "string" or value == ""' -or $runtime -notmatch 'return "unavailable"' -or $runtime -notmatch 'value:gsub\("\[\^ -~\]", "_"\):sub\(1, VERSION_DIAGNOSTIC_MAX_BYTES\)' -or $runtime -notmatch 'if not version_diagnostic_enabled or version_diagnostic_emitted then' -or $runtime -notmatch 'trace\("embedded_lua_version", runtime\.sanitize_embedded_version\(_VERSION\)\)' -or $runtime -notmatch 'trace_embedded_version_once\(\)') { throw 'embedded version diagnostic must be default-off, sanitized, bounded, and one-shot' }
if ([regex]::Matches($runtime, 'trace\("embedded_lua_version"').Count -ne 1) { throw 'embedded version diagnostic must emit at most one trace event' }
$discoveryContracts = Get-Content -Raw (Join-Path $extension 'tests/x4_discovery_contract.lua')
if ($discoveryContracts -notmatch 'sanitizes_embedded_version_without_exposing_unavailable_values' -or $discoveryContracts -notmatch 'string.rep\("x", 65\)' -or $discoveryContracts -notmatch 'sanitize_embedded_version\(nil\) == "unavailable"') { throw 'embedded version sanitization contract coverage is missing' }
if ($discoveryContracts -notmatch 'capability_probe_is_disabled_by_default' -or $discoveryContracts -notmatch 'capability_probe_emits_one_closed_privacy_safe_vector' -or $discoveryContracts -notmatch 'capability_probe_suppresses_a_failed_trace_write_after_one_attempt') { throw 'capability vector contract coverage is missing' }
if ($runtime -notmatch 'TRACE_CONFIG_MODULE' -or $runtime -notmatch 'attempt_id=' -or $runtime -notmatch 'hop=lua' -or $runtime -notmatch 'lifecycle_unavailable' -or $runtime -notmatch 'named_pipes.Interface' -or $runtime -notmatch 'pipes._Write_Pipe_Raw' -or $runtime -notmatch 'pipes.Disconnect_Pipe' -or $runtime -match 'pipes.Write_Pipe') { throw 'correlated Lua trace or explicit reconnect pipe adapter contract is missing' }
if ($runtime -notmatch 'LOCAL_MODULE_PREFIX = "live_galaxy/lua/"' -or $runtime -notmatch 'require\(LOCAL_MODULE_PREFIX \.\. name\)' -or $runtime -match 'return require\(name\)' -or $runtime -notmatch 'live_galaxy_x4_discovery' -or $runtime -notmatch 'live_galaxy_telemetry' -or $runtime -match 'runtime_probe' -or $runtime -match 'payload\("observation",[^\r\n]*sector:live_galaxy') { throw 'runtime extension-relative module loading, discovery-to-telemetry linkage, or fixed observation retirement check failed' }
$discoverySource = Get-Content -Raw $discovery
if ($discoverySource -notmatch 'capability_probe_enabled == true' -or $discoverySource -notmatch 'MAX_PROBE_ATTEMPT_BYTES = 64' -or $discoverySource -notmatch 'event=capability_vector metadata_type=' -or $discoverySource -notmatch 'first_cargo_ware_limit=' -or $discoverySource -notmatch 'probe_consumed = true' -or $discoverySource -match 'GetComponents\(') { throw 'capability vector must remain default-off, one-shot, closed, and free of component enumeration' }
$telemetry = Get-Content -Raw (Join-Path $extension 'lua/live_galaxy_telemetry.lua')
if ($telemetry -notmatch 'require\("live_galaxy/lua/live_galaxy_normalize"\)') { throw 'telemetry normalizer must use the extension-relative loader path' }
if (-not (Test-Path (Join-Path $PSScriptRoot '01-capture-trace.ps1'))) { throw 'trace capture helper is missing' }
if (-not (Test-Path (Join-Path $root 'target/debug/x4-bridge.exe'))) { throw 'bridge binary is missing' }
if ($VerifyPackageOnly) { exit 0 }
if (Get-Process -Name 'X4' -ErrorAction SilentlyContinue) { throw 'refusing installation while X4 is running' }
throw 'installation is intentionally deferred; use -VerifyPackageOnly'
