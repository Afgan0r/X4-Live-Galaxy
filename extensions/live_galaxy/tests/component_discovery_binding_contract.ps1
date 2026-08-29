$ErrorActionPreference = 'Stop'

$module = Join-Path (Split-Path -Parent $PSScriptRoot) 'lua/live_galaxy_component_discovery.lua'
if (-not (Test-Path -LiteralPath $module -PathType Leaf)) {
    throw "Missing component discovery adapter: $module"
}

$source = Get-Content -LiteralPath $module -Raw
foreach ($pattern in @(
    'C\.GetNumAllFactionStations',
    'C\.GetAllFactionStations',
    'UniverseID\[\?\]',
    'tonumber\(raw_count\)',
    'tonumber\(raw_filled\)',
    'for index = 0, count - 1 do',
    'to_component',
    'ConvertStringToLuaID',
    'buffer\[index\]',
    'ConvertIDTo64Bit',
    'GetComponentData\(component, "owner", "sector"\)',
    'C\.GetPeopleCapacity',
    'pcall'
)) {
    if ($source -notmatch $pattern) { throw "Missing protected binding: $pattern" }
}
