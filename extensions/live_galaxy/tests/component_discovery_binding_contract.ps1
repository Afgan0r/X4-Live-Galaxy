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
    'to_component64 = function\(_, component\) return globals\.ConvertIDTo64Bit\(component\) end',
    'GetComponentData\(component, "owner", "sector"\)',
    'get_people_capacity = function\(_, component64\)',
    'C\.GetPeopleCapacity\(component64, "", false\)',
    'pcall'
)) {
    if ($source -notmatch $pattern) { throw "Missing protected binding: $pattern" }
}

$conversionOffset = $source.IndexOf('local id_ok, component64 = pcall(api.to_component64')
$capacityOffset = $source.IndexOf('api.get_people_capacity, api, candidate.component64')
if ($conversionOffset -lt 0 -or $capacityOffset -lt 0 -or $conversionOffset -ge $capacityOffset) {
    throw 'Component 64-bit conversion must precede the native capacity call.'
}
