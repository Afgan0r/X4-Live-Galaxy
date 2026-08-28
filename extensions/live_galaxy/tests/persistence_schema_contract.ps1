$ErrorActionPreference = 'Stop'

$extensionRoot = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $extensionRoot 'checkpoint_schema.json'
$xmlPath = Join-Path $extensionRoot 'md/live_galaxy_persistence.xml'
$evidencePath = Join-Path (Split-Path -Parent (Split-Path -Parent $extensionRoot)) '.planning/phases/04-persistent-full-faction-minds/04-X4-PERSISTENCE-EVIDENCE.md'

foreach ($requiredPath in @($manifestPath, $xmlPath)) {
    if (-not (Test-Path -LiteralPath $requiredPath)) {
        throw "Required persistence contract artifact is missing: $requiredPath"
    }
}

if (-not (Test-Path -LiteralPath $evidencePath)) {
    throw "Required Phase 7 persistence evidence protocol is missing: $evidencePath"
}

$manifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
[xml]$xml = Get-Content -Raw -LiteralPath $xmlPath

if ($manifest.root_cue -ne 'live_galaxy_persistence_root') {
    throw 'The checkpoint manifest must name the stable persistence root cue.'
}

if ($manifest.variable_name -ne 'live_galaxy_checkpoint') {
    throw 'The checkpoint manifest must name the extension-scoped checkpoint variable.'
}

$requiredFields = @(
    'schema_version',
    'game_protocol_identity',
    'sequence',
    'integrity_hash',
    'compatibility_status',
    'x4_restart_required',
    'payload'
)

if (@($manifest.required_fields).Count -ne $requiredFields.Count -or
    @($manifest.required_fields | Where-Object { $_ -notin $requiredFields }).Count -ne 0) {
    throw 'The checkpoint manifest must declare exactly the canonical envelope fields.'
}

$rootCue = @($xml.mdscript.cues.cue | Where-Object { $_.name -eq $manifest.root_cue })
if ($rootCue.Count -ne 1) {
    throw 'The MD script must contain exactly one stable persistence root cue.'
}

if ($rootCue[0].instantiate -eq 'true') {
    throw 'The persistence root cue must remain static and may not be instantiated.'
}

$storageActions = @($rootCue.actions.set_value)
if ($storageActions.Count -ne 1 -or $storageActions[0].name -ne ('$' + $manifest.variable_name)) {
    throw 'The MD root cue must declare exactly one extension-scoped checkpoint variable.'
}

$initialEnvelope = $storageActions[0].exact -replace '^''|''$', '' | ConvertFrom-Json
foreach ($field in $requiredFields) {
    if ($null -eq $initialEnvelope.PSObject.Properties[$field]) {
        throw "The MD initial envelope is missing $field."
    }
}

if ($initialEnvelope.schema_version -ne $manifest.schema_version -or
    $initialEnvelope.game_protocol_identity -ne $manifest.game_protocol_identity) {
    throw 'The MD initial envelope must match the manifest schema and protocol identities.'
}

$forbiddenTerms = '(?i)model|report|acknowledg|pipe|lua|raise_lua_event|create_ship|set_relation|start_mission'
if ((Get-Content -Raw -LiteralPath $xmlPath) -match $forbiddenTerms) {
    throw 'The MD persistence contract may not include model, return-channel, or game-mutation behavior.'
}

$evidence = Get-Content -Raw -LiteralPath $evidencePath
$requiredEvidenceTerms = @(
    'Documented',
    'Verified locally',
    'Pending-X4',
    'Observed in X4',
    'Phase 7',
    'Creative Custom',
    'payload',
    'interruption',
    'save/load',
    'reconnect',
    'X4 build',
    'extension/mod set',
    'scenario',
    'elapsed game time',
    'elapsed real time',
    'checkpoint sequence/hash',
    'expected result',
    'reread result',
    'bounded diagnostics'
)

foreach ($term in $requiredEvidenceTerms) {
    if ($evidence -notmatch [regex]::Escape($term)) {
        throw "The Phase 7 evidence protocol is missing required category: $term"
    }
}

if ($evidence -notmatch '(?i)all runtime properties.*pending-X4') {
    throw 'The evidence protocol must keep all runtime properties pending-X4.'
}

if ($evidence -notmatch '(?i)Observed in X4\s*(?:\||:)\s*None') {
    throw 'The evidence protocol must not claim an observed X4 result.'
}

Write-Output 'Persistence schema contract passed.'
