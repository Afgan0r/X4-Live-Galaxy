param([string]$ExtensionRoot = (Split-Path -Parent $PSScriptRoot))

$ErrorActionPreference = 'Stop'

$manifestPath = Join-Path $extensionRoot 'checkpoint_schema.json'
$xmlPath = Join-Path $extensionRoot 'md/live_galaxy_persistence.xml'

foreach ($requiredPath in @($manifestPath, $xmlPath)) {
    if (-not (Test-Path -LiteralPath $requiredPath)) {
        throw "Required persistence contract artifact is missing: $requiredPath"
    }
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
    @($manifest.required_fields | Select-Object -Unique).Count -ne $requiredFields.Count -or
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

Write-Output 'Persistence schema contract passed.'
