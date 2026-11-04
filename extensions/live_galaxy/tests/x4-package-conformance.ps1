[CmdletBinding()]
param([string]$ExtensionRoot = (Split-Path -Parent $PSScriptRoot))

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Read-ProductXml([string]$Text) {
    $document = [Xml.XmlDocument]::new()
    $document.XmlResolver = $null
    $document.LoadXml($Text)
    return $document
}

function Assert-Registration([xml]$Content, [xml]$Ui, [string]$Root) {
    $contentDependencies = @($Content.DocumentElement.SelectNodes('./dependency'))
    if ($Content.DocumentElement.LocalName -cne 'content' -or
        $Content.DocumentElement.GetAttribute('id') -cne 'live_galaxy') {
        throw 'INVALID_PACKAGE_IDENTITY'
    }
    if ($contentDependencies.Count -ne 0) { throw 'CARRIER_A_DEPENDENCY_FORBIDDEN' }
    if ($Ui.DocumentElement.LocalName -cne 'addon' -or
        $Ui.DocumentElement.GetAttribute('name') -cne 'live_galaxy') { throw 'INVALID_ADDON_IDENTITY' }
    $menus = @($Ui.DocumentElement.SelectNodes('./environment') | Where-Object { $_.GetAttribute('type') -ceq 'menus' })
    if ($menus.Count -ne 1) { throw 'MISSING_MENUS_ENVIRONMENT' }
    $dependencies = @($menus[0].SelectNodes('./dependency'))
    if ($dependencies.Count -ne 0) { throw 'CARRIER_A_UI_DEPENDENCY_FORBIDDEN' }
    $files = @($menus[0].SelectNodes('./file'))
    if ($files.Count -ne 1 -or $files[0].GetAttribute('name') -cne 'lua/live_galaxy_runtime.lua') {
        throw 'WRONG_ENTRYPOINT'
    }
    if (-not (Test-Path -LiteralPath (Join-Path $Root 'lua/live_galaxy_runtime.lua') -PathType Leaf)) {
        throw 'MISSING_ENTRYPOINT'
    }
}

function Assert-Rejected([scriptblock]$Check, [string]$Expected) {
    try { & $Check }
    catch {
        if ($_.Exception.Message -notmatch $Expected) { throw }
        return
    }
    throw "Expected rejection: $Expected"
}

function Assert-LifecycleBoundaries([xml]$Document) {
    if ($Document.SelectNodes('//cue[@checkinterval or @instantiate]').Count -ne 0 -or
        $Document.SelectNodes('//raise_lua_event[@param="''telemetry_tick''"]').Count -ne 0) {
        throw 'MD_PERIODIC_PUMP_FORBIDDEN'
    }
    foreach ($boundary in @('game_start', 'game_loaded')) {
        $cue = $Document.SelectSingleNode("//cue[@name='live_galaxy_$boundary']")
        if ($null -eq $cue -or $cue.SelectNodes('./cues/cue').Count -ne 0 -or
            $cue.SelectNodes("./actions/raise_lua_event[@name=`"'live_galaxy_observation'`" and @param=`"'telemetry_$boundary'`"]").Count -ne 1) {
            throw 'LIFECYCLE_BOUNDARY_MUST_BE_ONE_SHOT'
        }
        $conditions = $cue.SelectNodes('./conditions')
        if ($conditions.Count -ne 1 -or $conditions[0].Attributes.Count -ne 0 -or
            $conditions[0].SelectNodes('./*').Count -ne 1) {
            throw 'LIFECYCLE_TRIGGER_MISMATCH'
        }
        $trigger = $conditions[0].SelectSingleNode('./*')
        if ($trigger.SelectNodes('./*').Count -ne 0 -or
            ($boundary -ceq 'game_start' -and
                ($trigger.LocalName -cne 'event_cue_signalled' -or $trigger.Attributes.Count -ne 1 -or
                    $trigger.GetAttribute('cue') -cne 'md.Setup.GameStart')) -or
            ($boundary -ceq 'game_loaded' -and
                ($trigger.LocalName -cne 'event_game_loaded' -or $trigger.Attributes.Count -ne 0))) {
            throw 'LIFECYCLE_TRIGGER_MISMATCH'
        }
    }
}

function Assert-TelemetryOnly([xml[]]$Documents, [string[]]$LuaTexts) {
    foreach ($document in $Documents) {
        $actions = @($document.SelectNodes('//actions/*'))
        foreach ($action in $actions) {
            if ($action.LocalName -cne 'raise_lua_event') {
                throw "FORBIDDEN_MUTATION_NODE:$($action.LocalName)"
            }
        }
        foreach ($comment in @($document.SelectNodes('//comment()'))) {
            [void]$comment.ParentNode.RemoveChild($comment)
        }
        if ($document.OuterXml -match '(?i)sn_mod_support_apis|carrier[_-]?a') {
            throw 'FORBIDDEN_CARRIER_A_REFERENCE'
        }
    }
    if (($LuaTexts -join "`n") -match '(?i)sn_mod_support_apis|carrier[_-]?a') {
        throw 'FORBIDDEN_CARRIER_A_REFERENCE'
    }
    if (($LuaTexts -join "`n") -match 'require\s*\(\s*["'']live_galaxy/lua/') {
        throw 'SLASH_QUALIFIED_LOCAL_IMPORT_FORBIDDEN'
    }
}

$content = Read-ProductXml (Get-Content -LiteralPath (Join-Path $ExtensionRoot 'content.xml') -Raw)
$ui = Read-ProductXml (Get-Content -LiteralPath (Join-Path $ExtensionRoot 'ui.xml') -Raw)
$xmlFiles = @(Get-ChildItem -LiteralPath $ExtensionRoot -Recurse -File -Filter '*.xml' |
    Where-Object { $_.FullName -notmatch '[\\/]tests[\\/]' })
foreach ($file in $xmlFiles) { $null = Read-ProductXml (Get-Content -LiteralPath $file.FullName -Raw) }
Assert-Registration $content $ui $ExtensionRoot
$mdFiles = @(Get-Item -LiteralPath (Join-Path $ExtensionRoot 'md/live_galaxy_observation.xml'))
$mdDocuments = @($mdFiles | ForEach-Object { Read-ProductXml (Get-Content -LiteralPath $_.FullName -Raw) })
Assert-LifecycleBoundaries $mdDocuments[0]
$luaTexts = @(Get-ChildItem -LiteralPath (Join-Path $ExtensionRoot 'lua') -File -Filter '*.lua' |
    ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw })
Assert-TelemetryOnly $mdDocuments $luaTexts

# A few product negatives run in-process; no parser corpus or subprocess per fixture.
Assert-Rejected { Read-ProductXml '<content>' } 'LoadXml'
$invalid = $content.CloneNode($true)
$invalid.DocumentElement.SetAttribute('id', 'wrong')
Assert-Rejected { Assert-Registration $invalid $ui $ExtensionRoot } '^INVALID_PACKAGE_IDENTITY$'
$invalid = $ui.CloneNode($true)
$dependency = $invalid.CreateElement('dependency')
$dependency.SetAttribute('name', 'sn_mod_support_apis')
[void]$invalid.DocumentElement.SelectSingleNode('./environment').AppendChild($dependency)
Assert-Rejected { Assert-Registration $content $invalid $ExtensionRoot } '^CARRIER_A_UI_DEPENDENCY_FORBIDDEN$'
Assert-Rejected { Assert-Registration $content $ui (Join-Path $ExtensionRoot 'missing-entrypoint-fixture') } '^MISSING_ENTRYPOINT$'
$invalidMd = Read-ProductXml '<mdscript><cues><cue><actions><create_ship /></actions></cue></cues></mdscript>'
Assert-Rejected { Assert-TelemetryOnly @($invalidMd) @() } '^FORBIDDEN_MUTATION_NODE:create_ship$'
Assert-Rejected { Assert-TelemetryOnly @() @('require("sn_mod_support_apis")') } '^FORBIDDEN_CARRIER_A_REFERENCE$'
Assert-Rejected { Assert-TelemetryOnly @() @('require("live_galaxy/lua/module")') } '^SLASH_QUALIFIED_LOCAL_IMPORT_FORBIDDEN$'
$invalid = $mdDocuments[0].CloneNode($true)
$invalid.SelectSingleNode('//cue').SetAttribute('checkinterval', '50ms')
Assert-Rejected { Assert-LifecycleBoundaries $invalid } '^MD_PERIODIC_PUMP_FORBIDDEN$'
$invalid = $mdDocuments[0].CloneNode($true)
$invalid.SelectSingleNode('//cue/actions/raise_lua_event').SetAttribute('param', "'telemetry_tick'")
Assert-Rejected { Assert-LifecycleBoundaries $invalid } '^MD_PERIODIC_PUMP_FORBIDDEN$'
$invalid = $mdDocuments[0].CloneNode($true)
$invalid.SelectSingleNode('//cue/actions/raise_lua_event').SetAttribute('param', "'unknown'")
Assert-Rejected { Assert-LifecycleBoundaries $invalid } '^LIFECYCLE_BOUNDARY_MUST_BE_ONE_SHOT$'
foreach ($boundary in @('game_start', 'game_loaded')) {
    foreach ($mutation in @('missing_conditions', 'wrong_event', 'extra_event', 'wrong_shape')) {
        $invalid = $mdDocuments[0].CloneNode($true)
        $cue = $invalid.SelectSingleNode("//cue[@name='live_galaxy_$boundary']")
        $conditions = $cue.SelectSingleNode('./conditions')
        $trigger = $conditions.SelectSingleNode('./*')
        switch ($mutation) {
            'missing_conditions' { [void]$cue.RemoveChild($conditions) }
            'wrong_event' { [void]$conditions.ReplaceChild($invalid.CreateElement('event_wrong_boundary'), $trigger) }
            'extra_event' { [void]$conditions.AppendChild($trigger.CloneNode($true)) }
            'wrong_shape' { $trigger.SetAttribute('cue', 'md.Setup.WrongBoundary') }
        }
        Assert-Rejected { Assert-LifecycleBoundaries $invalid } '^LIFECYCLE_TRIGGER_MISMATCH$'
    }
}
Write-Output "XML package: $($xmlFiles.Count) well-formed files, registration and 18 rejection checks passed."
