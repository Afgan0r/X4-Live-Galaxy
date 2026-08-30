[CmdletBinding()]
param(
    [ValidateSet('packaged-path', 'all')]
    [string]$Case = 'all'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$commandPath = Join-Path $root 'tools/x4-verification/x4-package-conformance.ps1'
$contractPath = Join-Path $root 'tools/x4-verification/contracts/package-conformance.v1.json'
$fixturePath = Join-Path $root 'tools/x4-verification/fixtures/package-negative-fixtures.v1.json'
$dossierPath = Join-Path $root 'tools/x4-verification/contracts/dossier.v1.json'
$registryPath = Join-Path $root 'tools/x4-verification/contracts/known-failures.v1.json'
$coveragePath = Join-Path $root 'tools/x4-verification/contracts/coverage.v1.json'
$admissionFixturePath = Join-Path $root 'tools/x4-verification/fixtures/negative-fixtures.v1.json'
$admissionPath = Join-Path $root 'tools/x4-verification/x4-admission.ps1'
$packageRoot = Join-Path $root 'extensions/live_galaxy'
$aggregateRunner = Join-Path $root 'extensions/live_galaxy/tests/run_contracts.ps1'

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Invoke-JsonCommand([string]$Path, [string[]]$Arguments, [int]$ExpectedExitCode) {
    $output = & pwsh -NoProfile -File $Path @Arguments 2>&1
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne $ExpectedExitCode) {
        throw "Unexpected exit code $exitCode (expected $ExpectedExitCode): $($output -join [Environment]::NewLine)"
    }
    $jsonLine = @($output | Where-Object { $_ -isnot [System.Management.Automation.ErrorRecord] })[-1]
    try {
        return ($jsonLine | ConvertFrom-Json -Depth 32)
    }
    catch {
        throw "Command did not return JSON: $($output -join [Environment]::NewLine)"
    }
}

function New-FixturePackage($Fixture, [string]$Destination) {
    $null = New-Item -ItemType Directory -Path $Destination -Force
    foreach ($file in $Fixture.files.PSObject.Properties) {
        $target = Join-Path $Destination $file.Name
        $parent = Split-Path -Parent $target
        $null = New-Item -ItemType Directory -Path $parent -Force
        Set-Content -LiteralPath $target -Value ([string]$file.Value) -NoNewline -Encoding utf8
    }
}

function New-SyntaxPackage([string]$Destination, [string]$EntrypointSource, [hashtable]$AdditionalFiles = @{}) {
    $fixture = [pscustomobject]@{ files = [pscustomobject]@{
        'content.xml' = '<content id="live_galaxy"><dependency id="ws_2042901274" /></content>'
        'ui.xml' = '<addon name="live_galaxy"><environment type="menus"><dependency name="sn_mod_support_apis" /><file name="lua/live_galaxy_runtime.lua" /></environment></addon>'
        'lua/live_galaxy_runtime.lua' = $EntrypointSource
    } }
    foreach ($entry in $AdditionalFiles.GetEnumerator()) {
        $fixture.files | Add-Member -NotePropertyName $entry.Key -NotePropertyValue $entry.Value
    }
    New-FixturePackage $fixture $Destination
}

function Assert-NoAbsolutePath($Value) {
    $json = $Value | ConvertTo-Json -Compress -Depth 32
    Assert-True ($json -notmatch '[A-Za-z]:[\\/]') 'Result leaked a Windows absolute path.'
    Assert-True ($json -notmatch '(?<![A-Za-z0-9_])/(?:home|Users|tmp|var)/') 'Result leaked an absolute host path.'
}

function Invoke-Conformance([string]$CandidateRoot, [int]$ExpectedExitCode) {
    return Invoke-JsonCommand $commandPath @(
        '-PackageRoot', $CandidateRoot,
        '-ContractPath', $contractPath,
        '-DossierPath', $dossierPath,
        '-CoveragePath', $coveragePath
    ) $ExpectedExitCode
}

function Test-PackagedPath {
    foreach ($required in @($commandPath, $contractPath, $fixturePath)) {
        Assert-True (Test-Path -LiteralPath $required -PathType Leaf) "Missing conformance artifact: $required"
    }

    $result = Invoke-Conformance $packageRoot 0
    Assert-True ($result.schema_version -eq 'x4-package-conformance-result.v1') 'Unexpected result schema.'
    Assert-True ($result.verdict -eq 'conformant') 'Repository package was not conformant.'
    Assert-True ($result.classification -eq 'production-faithful') 'Repository package was not production-faithful.'
    Assert-True ($result.evidence_level -eq 'packaged-static') 'Static package evidence was overstated.'
    Assert-True ($result.entrypoint -eq 'lua/live_galaxy_runtime.lua') 'Registered entrypoint was not followed.'
    Assert-True ($result.native_binding_path -eq 'lua/live_galaxy_component_discovery.lua') 'Native binding acquisition path was not followed.'
    Assert-True (@($result.import_graph) -contains 'lua/live_galaxy_x4_discovery.lua') 'Transitive X4 discovery import is absent.'
    Assert-True (@($result.import_graph) -contains 'lua/live_galaxy_telemetry.lua') 'Transitive telemetry import is absent.'
    Assert-True (@($result.import_graph) -contains 'lua/live_galaxy_normalize.lua') 'Transitive normalizer import is absent.'
    Assert-True ($result.dossier_digest -match '^[a-f0-9]{64}$') 'Dossier digest is absent.'
    Assert-True ($result.coverage_digest -match '^[a-f0-9]{64}$') 'Coverage digest is absent.'
    Assert-True ($result.graph_digest -match '^[a-f0-9]{64}$') 'Graph digest is absent.'
    Assert-NoAbsolutePath $result

    $admission = Invoke-JsonCommand $admissionPath @(
        '-DossierPath', $dossierPath,
        '-RegistryPath', $registryPath,
        '-CoveragePath', $coveragePath,
        '-FixturePath', $admissionFixturePath,
        '-ValidateFixture'
    ) 0
    Assert-True ($admission.verdict -eq 'validation-passed') 'Independent Plan 01 fixture validation did not pass.'

    $bundle = Get-Content -LiteralPath $fixturePath -Raw | ConvertFrom-Json -Depth 64
    Assert-True ($bundle.schema_version -eq 'x4-package-negative-fixtures.v1') 'Unexpected fixture schema.'
    $expectedIds = @(
        'missing-registration',
        'wrong-entrypoint',
        'unresolved-import',
        'import-cycle',
        'root-escape',
        'alternate-binding-source',
        'graph-depth-exceeded',
        'graph-size-exceeded',
        'bare-import',
        'test-only-loader'
    )
    Assert-True ((@($bundle.fixtures.id | Sort-Object) -join '|') -eq (($expectedIds | Sort-Object) -join '|')) 'Negative fixture set is incomplete.'

    $tempRoot = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
    $scratch = Join-Path $tempRoot ("live-galaxy-conformance-" + [guid]::NewGuid().ToString('N'))
    try {
        foreach ($fixture in $bundle.fixtures) {
            $candidate = Join-Path $scratch $fixture.id
            New-FixturePackage $fixture $candidate
            $negative = Invoke-Conformance $candidate 1
            Assert-True ($negative.verdict -eq 'non-conformant') "Fixture '$($fixture.id)' did not fail closed."
            Assert-True (@($negative.reason_codes) -contains $fixture.expected_reason_code) "Fixture '$($fixture.id)' returned '$(@($negative.reason_codes) -join ',')', expected '$($fixture.expected_reason_code)'."
            Assert-True ($negative.classification -eq $fixture.expected_classification) "Fixture '$($fixture.id)' returned the wrong classification."
            Assert-NoAbsolutePath $negative
        }

        $staticPackage = Join-Path $scratch 'lexer-static-forms'
        New-SyntaxPackage $staticPackage @'
-- require("test/ignored_comment")
local ignored = [[require("test/ignored_long_string")]]
local PREFIX = "live_galaxy/lua/"
local ffi = require(
    "ffi"
)
local C = ffi.C
local dependency = require(PREFIX .. "dependency")
'@ @{ 'lua/dependency.lua' = 'return {}' }
        $staticResult = Invoke-Conformance $staticPackage 0
        Assert-True ($staticResult.verdict -eq 'conformant') 'Lexer rejected supported multiline/concatenated imports or scanned comments/long strings.'

        $aliasPackage = Join-Path $scratch 'lexer-alias'
        New-SyntaxPackage $aliasPackage 'local r = require; local ffi = r("ffi"); local C = ffi.C'
        $aliasResult = Invoke-Conformance $aliasPackage 1
        Assert-True (@($aliasResult.reason_codes) -contains 'REQUIRE_ALIAS_UNSUPPORTED') 'Require alias did not fail closed.'

        $dynamicPackage = Join-Path $scratch 'lexer-dynamic'
        New-SyntaxPackage $dynamicPackage 'local name = get_name(); local ffi = require(name); local C = ffi.C'
        $dynamicResult = Invoke-Conformance $dynamicPackage 1
        Assert-True (@($dynamicResult.reason_codes) -contains 'DYNAMIC_REQUIRE') 'Dynamic require did not fail closed.'

        $outside = Join-Path $scratch 'outside-target'
        $null = New-Item -ItemType Directory -Path $outside -Force
        Set-Content -LiteralPath (Join-Path $outside 'payload.lua') -Value 'return {}' -NoNewline -Encoding utf8
        $reparsePackage = Join-Path $scratch 'reparse-escape'
        New-SyntaxPackage $reparsePackage 'local ffi = require("ffi"); local C = ffi.C; local escaped = require("live_galaxy/lua/escape/payload")'
        $linkPath = Join-Path $reparsePackage 'lua/escape'
        $linkType = if ($IsWindows) { 'Junction' } else { 'SymbolicLink' }
        $null = New-Item -ItemType $linkType -Path $linkPath -Target $outside
        $reparseResult = Invoke-Conformance $reparsePackage 1
        Assert-True (@($reparseResult.reason_codes) -contains 'REPARSE_POINT_ESCAPE') 'Reparse-point package escape was not rejected.'
    }
    finally {
        if (Test-Path -LiteralPath $scratch) {
            $resolvedScratch = [System.IO.Path]::GetFullPath($scratch)
            if (-not $resolvedScratch.StartsWith($tempRoot, [StringComparison]::OrdinalIgnoreCase)) {
                throw "Refusing to remove fixture root outside the system temporary directory."
            }
            Remove-Item -LiteralPath $scratch -Recurse -Force
        }
    }
}

function Test-AggregateRegistration {
    $output = & pwsh -NoProfile -File $aggregateRunner -Suite x4-package-conformance 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Aggregate package conformance failed: $($output -join [Environment]::NewLine)"
    }
    Assert-True (@($output) -contains 'package conformance contract passed: packaged-path') 'Aggregate runner did not execute the production package contract.'
}

switch ($Case) {
    'packaged-path' { Test-PackagedPath }
    'all' {
        Test-PackagedPath
        Test-AggregateRegistration
    }
}

Write-Output "package conformance contract passed: $Case"
