[CmdletBinding()]
param(
    [ValidateSet('all', 'lua', 'component_discovery', 'x4_discovery', 'telemetry', 'scheduler', 'syntax', 'xml')]
    [string]$Suite = 'all',
    [string]$Filter,
    [string]$ExtensionRoot = (Split-Path -Parent $PSScriptRoot)
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$aggregate = [Diagnostics.Stopwatch]::StartNew()
$result = 1

function Invoke-Stage([string]$Name, [string]$Executable, [string[]]$Arguments) {
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $code = 1
    try {
        & $Executable @Arguments
        $code = $LASTEXITCODE
    }
    finally {
        $timer.Stop()
        Write-Output "STAGE $Name exit=$code elapsed_seconds=$($timer.Elapsed.TotalSeconds.ToString('R', [Globalization.CultureInfo]::InvariantCulture))"
    }
    if ($code -ne 0) { throw "$Name failed with exit code $code." }
}

Push-Location $root
try {
    if ($Suite -ne 'xml') {
        $lock = Get-Content -LiteralPath (Join-Path $root 'tools/lua-runner.lock.json') -Raw | ConvertFrom-Json
        $lua = Join-Path $root "$($lock.bustedDevelopment.rootRelativePath)/bin/lua.exe"
        $busted = Join-Path $root "$($lock.bustedDevelopment.treeRelativePath)/bin/busted.bat"
        if (-not (Test-Path -LiteralPath $lua) -or -not (Test-Path -LiteralPath $busted)) {
            throw 'Busted tooling is absent. Run tools/provision-lua.ps1 -WithBusted -CompilerPath <installed clang.exe> once.'
        }
        $version = (& $lua -v 2>&1 | Out-String).Trim()
        if ($LASTEXITCODE -ne 0 -or $version -notmatch "^$([regex]::Escape($lock.executableVersion))(\s|$)") {
            throw 'The development Lua runtime does not match lua-runner.lock.json.'
        }
        $arguments = @('--lpath=extensions/?.lua;extensions/live_galaxy/tests/?.lua')
        if ($Filter) { $arguments += @('--filter', $Filter) }
        $files = switch ($Suite) {
            'component_discovery' { 'component_discovery_contract.lua' }
            'x4_discovery' { 'x4_discovery_contract.lua'; 'module_loading_spec.lua'; $arguments += '--exclude-tags=syntax' }
            'telemetry' { 'telemetry_spec.lua' }
            'scheduler' { 'scheduler_contract.lua' }
            'syntax' { 'module_loading_spec.lua'; $arguments += '--tags=syntax' }
            default { 'component_discovery_contract.lua'; 'x4_discovery_contract.lua'; 'telemetry_spec.lua'; 'scheduler_contract.lua'; 'module_loading_spec.lua' }
        }
        $arguments += @($files | ForEach-Object { Join-Path $PSScriptRoot $_ })
        Invoke-Stage 'Busted' $busted $arguments
    }
    if ($Suite -in @('all', 'xml')) {
        Invoke-Stage 'XML' 'pwsh' @('-NoProfile', '-File', (Join-Path $PSScriptRoot 'x4-package-conformance.ps1'), '-ExtensionRoot', $ExtensionRoot)
        Invoke-Stage 'persistence-schema' 'pwsh' @('-NoProfile', '-File', (Join-Path $PSScriptRoot 'persistence_schema_contract.ps1'), '-ExtensionRoot', $ExtensionRoot)
    }
    $result = 0
}
catch { Write-Error $_ -ErrorAction Continue }
finally {
    Pop-Location
    $aggregate.Stop()
    Write-Output "AGGREGATE suite=$Suite exit=$result elapsed_seconds=$($aggregate.Elapsed.TotalSeconds.ToString('R', [Globalization.CultureInfo]::InvariantCulture))"
}
exit $result
