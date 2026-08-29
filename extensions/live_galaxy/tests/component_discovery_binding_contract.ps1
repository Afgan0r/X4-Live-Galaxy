$ErrorActionPreference = 'Stop'

$module = Join-Path (Split-Path -Parent $PSScriptRoot) 'lua/live_galaxy_component_discovery.lua'
$runtimeModule = Join-Path (Split-Path -Parent $PSScriptRoot) 'lua/live_galaxy_x4_discovery.lua'
$entryModule = Join-Path (Split-Path -Parent $PSScriptRoot) 'lua/live_galaxy_runtime.lua'
if (-not (Test-Path -LiteralPath $module -PathType Leaf)) {
    throw "Missing component discovery adapter: $module"
}
if (-not (Test-Path -LiteralPath $runtimeModule -PathType Leaf)) {
    throw "Missing X4 discovery adapter: $runtimeModule"
}

$source = Get-Content -LiteralPath $module -Raw
foreach ($pattern in @(
    'local ffi = require\("ffi"\)',
    'local C = ffi\.C',
    'C\.GetNumAllFactionStations',
    'C\.GetAllFactionStations',
    'UniverseID\[\?\]',
    'tonumber\(raw_count\)',
    'tonumber\(raw_filled\)',
    'for index = 0, count - 1 do',
    'to_component',
    'ConvertStringToLuaID',
    'buffer\[index\]',
    'to_component64 = function\(_, component\) return globals\.ConvertIDTo64Bit\(component\) end',
    'GetComponentData\(component, "owner", "sector"\)',
    'get_people_capacity = function\(_, component64\)',
    'C\.GetPeopleCapacity\(component64, "", false\)',
    'function adapter\.diagnostic_class\(\)',
    'metadata_unavailable',
    'owner_scope_mismatch',
    'capacity_invalid',
    'pcall'
)) {
    if ($source -notmatch $pattern) { throw "Missing protected binding: $pattern" }
}
if ($source -match 'globals\.C\.') {
    throw 'Native FFI calls must use the module-local ffi.C binding.'
}

$conversionOffset = $source.IndexOf('local id_ok, component64 = pcall(api.to_component64')
$capacityOffset = $source.IndexOf('api.get_people_capacity, api, candidate.component64')
if ($conversionOffset -lt 0 -or $capacityOffset -lt 0 -or $conversionOffset -ge $capacityOffset) {
    throw 'Component 64-bit conversion must precede the native capacity call.'
}

$runtimeSource = Get-Content -LiteralPath $runtimeModule -Raw
if ($runtimeSource -notmatch 'require\("live_galaxy/lua/live_galaxy_component_discovery"\)' -or
    $runtimeSource -match 'require\("live_galaxy_component_discovery"\)') {
    throw 'Component discovery must use the extension-relative X4 module path.'
}
if (-not (Test-Path -LiteralPath $entryModule -PathType Leaf)) {
    throw "Missing runtime entrypoint: $entryModule"
}
$entrySource = Get-Content -LiteralPath $entryModule -Raw
if ($entrySource -notmatch 'if trace_enabled and err == "facts_unsupported"' -or
    $entrySource -notmatch 'component_discovery_class' -or
    $entrySource -notmatch 'DISCOVERY_DIAGNOSTIC_CLASSES\[diagnostic_class\]') {
    throw 'The component diagnostic must remain opt-in and allowlisted.'
}
