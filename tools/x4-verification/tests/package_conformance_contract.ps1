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
        '-FixturePath', $admissionFixturePath
    ) 0
    Assert-True ($admission.verdict -eq 'admissible') 'Independent Plan 01 admission did not accept its complete fixture.'

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

switch ($Case) {
    'packaged-path' { Test-PackagedPath }
    'all' { Test-PackagedPath }
}

Write-Output "package conformance contract passed: $Case"
