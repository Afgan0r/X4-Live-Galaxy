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
        $Content.DocumentElement.GetAttribute('id') -cne 'live_galaxy' -or
        @($contentDependencies | Where-Object { $_.GetAttribute('id') -ceq 'ws_2042901274' }).Count -ne 1) {
        throw 'INVALID_PACKAGE_IDENTITY'
    }
    if ($Ui.DocumentElement.LocalName -cne 'addon' -or
        $Ui.DocumentElement.GetAttribute('name') -cne 'live_galaxy') { throw 'INVALID_ADDON_IDENTITY' }
    $menus = @($Ui.DocumentElement.SelectNodes('./environment') | Where-Object { $_.GetAttribute('type') -ceq 'menus' })
    if ($menus.Count -ne 1) { throw 'MISSING_MENUS_ENVIRONMENT' }
    $dependencies = @($menus[0].SelectNodes('./dependency') | Where-Object { $_.GetAttribute('name') -ceq 'sn_mod_support_apis' })
    if ($dependencies.Count -ne 1) { throw 'MISSING_UI_DEPENDENCY' }
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

$content = Read-ProductXml (Get-Content -LiteralPath (Join-Path $ExtensionRoot 'content.xml') -Raw)
$ui = Read-ProductXml (Get-Content -LiteralPath (Join-Path $ExtensionRoot 'ui.xml') -Raw)
$xmlFiles = @(Get-ChildItem -LiteralPath $ExtensionRoot -Recurse -File -Filter '*.xml' |
    Where-Object { $_.FullName -notmatch '[\\/]tests[\\/]' })
foreach ($file in $xmlFiles) { $null = Read-ProductXml (Get-Content -LiteralPath $file.FullName -Raw) }
Assert-Registration $content $ui $ExtensionRoot

# A few product negatives run in-process; no parser corpus or subprocess per fixture.
Assert-Rejected { Read-ProductXml '<content>' } 'LoadXml'
$invalid = $content.CloneNode($true)
$invalid.DocumentElement.SetAttribute('id', 'wrong')
Assert-Rejected { Assert-Registration $invalid $ui $ExtensionRoot } '^INVALID_PACKAGE_IDENTITY$'
$invalid = $ui.CloneNode($true)
$invalid.DocumentElement.SelectSingleNode('./environment/dependency').SetAttribute('name', 'wrong')
Assert-Rejected { Assert-Registration $content $invalid $ExtensionRoot } '^MISSING_UI_DEPENDENCY$'
Assert-Rejected { Assert-Registration $content $ui (Join-Path $ExtensionRoot 'missing-entrypoint-fixture') } '^MISSING_ENTRYPOINT$'
Write-Output "XML package: $($xmlFiles.Count) well-formed files, registration and 4 rejection checks passed."
