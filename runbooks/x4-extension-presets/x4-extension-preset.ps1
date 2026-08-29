[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidateSet('Capture', 'ApplyTest', 'Restore', 'Status')]
    [string]$Action,

    [string]$ConfigPath,

    [string]$StateRoot,

    [string]$ExtensionsRoot = 'F:\SteamLibrary\steamapps\common\X4 Foundations\extensions',

    [string[]]$EnabledExtensionId = @('live_galaxy', 'ws_2042901274'),

    [switch]$Force
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Resolve-X4ConfigPath {
    param([string]$RequestedPath)

    if ($RequestedPath) {
        return [System.IO.Path]::GetFullPath($RequestedPath)
    }

    $profilesRoot = Join-Path $env:USERPROFILE 'Documents\Egosoft\X4'
    $matches = @(
        Get-ChildItem -LiteralPath $profilesRoot -Directory -ErrorAction Stop |
            Where-Object { $_.Name -match '^\d+$' } |
            ForEach-Object { Join-Path $_.FullName 'content.xml' } |
            Where-Object { Test-Path -LiteralPath $_ }
    )

    if ($matches.Count -ne 1) {
        throw "Expected exactly one X4 profile content.xml beneath $profilesRoot; pass -ConfigPath explicitly."
    }

    return $matches[0]
}

function Assert-X4Stopped {
    if (Get-Process -Name 'X4' -ErrorAction SilentlyContinue) {
        throw 'X4 is running. Close it before capturing, applying, or restoring an extension preset.'
    }
}

function Get-Sha256 {
    param([Parameter(Mandatory)][string]$Path)

    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Write-AtomicText {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Content
    )

    $directory = Split-Path -Parent $Path
    $temporaryPath = Join-Path $directory ('.' + [System.IO.Path]::GetRandomFileName())
    try {
        [System.IO.File]::WriteAllText($temporaryPath, $Content, [System.Text.UTF8Encoding]::new($false))
        Move-Item -LiteralPath $temporaryPath -Destination $Path -Force
    }
    finally {
        if (Test-Path -LiteralPath $temporaryPath) {
            Remove-Item -LiteralPath $temporaryPath -Force
        }
    }
}

function Set-PrivateAcl {
    param(
        [Parameter(Mandatory)][string]$Path,
        [switch]$Directory
    )

    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent().Name
    $grant = if ($Directory) { "$($currentUser):(OI)(CI)F" } else { "$($currentUser):F" }
    & icacls.exe $Path /inheritance:r /grant:r $grant | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to protect retained preset artifact at $Path."
    }
}

function Read-Manifest {
    param([Parameter(Mandatory)][string]$ManifestPath)

    if (-not (Test-Path -LiteralPath $ManifestPath)) {
        throw "No captured preset manifest exists at $ManifestPath. Run Capture first."
    }

    return (Get-Content -LiteralPath $ManifestPath -Raw | ConvertFrom-Json)
}

function Get-InstalledThirdPartyExtensionIds {
    param([Parameter(Mandatory)][string]$Root)

    if (-not (Test-Path -LiteralPath $Root -PathType Container)) {
        throw "X4 extensions directory was not found at $Root."
    }

    $ids = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
    foreach ($directory in Get-ChildItem -LiteralPath $Root -Directory) {
        $contentPath = Join-Path $directory.FullName 'content.xml'
        if (-not (Test-Path -LiteralPath $contentPath -PathType Leaf)) { continue }
        [xml]$content = Get-Content -LiteralPath $contentPath -Raw
        $id = [string]$content.content.id
        if (-not [string]::IsNullOrWhiteSpace($id) -and -not $id.StartsWith('ego_', [System.StringComparison]::Ordinal)) {
            [void]$ids.Add($id)
        }
    }

    return $ids
}

$resolvedConfigPath = Resolve-X4ConfigPath -RequestedPath $ConfigPath
if (-not (Test-Path -LiteralPath $resolvedConfigPath)) {
    throw "X4 profile configuration was not found at $resolvedConfigPath."
}

if (-not $StateRoot) {
    $StateRoot = Join-Path $env:LOCALAPPDATA 'LiveGalaxy\x4-extension-presets'
}
$resolvedStateRoot = [System.IO.Path]::GetFullPath($StateRoot)
$backupPath = Join-Path $resolvedStateRoot 'baseline-content.xml'
$manifestPath = Join-Path $resolvedStateRoot 'baseline-manifest.json'
$resolvedExtensionsRoot = [System.IO.Path]::GetFullPath($ExtensionsRoot)

switch ($Action) {
    'Capture' {
        Assert-X4Stopped
        if ((Test-Path -LiteralPath $backupPath) -and -not $Force) {
            throw "A captured preset already exists at $backupPath. Use -Force only when intentionally replacing it."
        }

        New-Item -ItemType Directory -Path $resolvedStateRoot -Force | Out-Null
        Set-PrivateAcl -Path $resolvedStateRoot -Directory
        Copy-Item -LiteralPath $resolvedConfigPath -Destination $backupPath -Force
        Set-PrivateAcl -Path $backupPath
        $backupHash = Get-Sha256 -Path $backupPath
        $manifest = [ordered]@{
            artifact_id = 'live-galaxy-x4-extension-preset-baseline'
            config_path = $resolvedConfigPath
            backup_path = $backupPath
            sha256 = $backupHash
            captured_at_utc = [DateTime]::UtcNow.ToString('o')
        } | ConvertTo-Json
        Write-AtomicText -Path $manifestPath -Content $manifest
        Set-PrivateAcl -Path $manifestPath

        if ((Get-Sha256 -Path $backupPath) -ne $backupHash) {
            throw 'Captured preset verification failed: backup digest changed after capture.'
        }
        [pscustomobject]@{ action = $Action; config_path = $resolvedConfigPath; backup_path = $backupPath; sha256 = $backupHash }
    }
    'ApplyTest' {
        Assert-X4Stopped
        $manifest = Read-Manifest -ManifestPath $manifestPath
        if ((Get-Sha256 -Path $backupPath) -ne $manifest.sha256) {
            throw 'Captured preset verification failed: backup digest does not match its manifest.'
        }

        [xml]$configuration = Get-Content -LiteralPath $resolvedConfigPath -Raw
        $configuration.content.SetAttribute('sync', 'false')
        $enabledIds = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
        foreach ($id in $EnabledExtensionId) {
            if ([string]::IsNullOrWhiteSpace($id)) { throw 'Enabled extension IDs must be non-empty.' }
            [void]$enabledIds.Add($id)
        }

        $configuredIds = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
        foreach ($extension in @($configuration.content.extension)) {
            [void]$configuredIds.Add($extension.id)
        }
        $discoveredIds = Get-InstalledThirdPartyExtensionIds -Root $resolvedExtensionsRoot
        foreach ($discoveredId in $discoveredIds) {
            if (-not $configuredIds.Contains($discoveredId)) {
                $extension = $configuration.CreateElement('extension')
                $extension.SetAttribute('id', $discoveredId)
                $extension.SetAttribute('enabled', 'false')
                [void]$configuration.content.AppendChild($extension)
            }
        }

        foreach ($extension in @($configuration.content.extension)) {
            $enabled = if ($enabledIds.Contains($extension.id)) { 'true' } else { 'false' }
            $extension.SetAttribute('enabled', $enabled)
            [void]$enabledIds.Remove($extension.id)
        }
        foreach ($missingId in $enabledIds) {
            $extension = $configuration.CreateElement('extension')
            $extension.SetAttribute('id', $missingId)
            $extension.SetAttribute('enabled', 'true')
            [void]$configuration.content.AppendChild($extension)
        }

        Write-AtomicText -Path $resolvedConfigPath -Content $configuration.OuterXml
        [xml]$verified = Get-Content -LiteralPath $resolvedConfigPath -Raw
        if ($verified.content.sync -ne 'false') {
            throw 'Test preset verification failed: Workshop extension synchronization remains enabled.'
        }
        $actualEnabled = @($verified.content.extension | Where-Object { $_.enabled -eq 'true' } | ForEach-Object id)
        $unexpected = @($actualEnabled | Where-Object { -not ($EnabledExtensionId -contains $_) })
        $missing = @($EnabledExtensionId | Where-Object { -not ($actualEnabled -contains $_) })
        if ($unexpected.Count -gt 0 -or $missing.Count -gt 0) {
            throw 'Test preset verification failed: enabled extension set differs from the requested IDs.'
        }
        [pscustomobject]@{ action = $Action; config_path = $resolvedConfigPath; enabled_extension_ids = $actualEnabled }
    }
    'Restore' {
        Assert-X4Stopped
        $manifest = Read-Manifest -ManifestPath $manifestPath
        if ((Get-Sha256 -Path $backupPath) -ne $manifest.sha256) {
            throw 'Restore refused: backup digest does not match its manifest.'
        }
        Copy-Item -LiteralPath $backupPath -Destination $resolvedConfigPath -Force
        if ((Get-Sha256 -Path $resolvedConfigPath) -ne $manifest.sha256) {
            throw 'Restore verification failed: active configuration digest differs from captured baseline.'
        }
        [pscustomobject]@{ action = $Action; config_path = $resolvedConfigPath; restored_sha256 = $manifest.sha256 }
    }
    'Status' {
        [xml]$configuration = Get-Content -LiteralPath $resolvedConfigPath -Raw
        $enabled = @($configuration.content.extension | Where-Object { $_.enabled -eq 'true' } | ForEach-Object id)
        $backupHash = if (Test-Path -LiteralPath $backupPath) { Get-Sha256 -Path $backupPath } else { $null }
        [pscustomobject]@{
            config_path = $resolvedConfigPath
            x4_running = [bool](Get-Process -Name 'X4' -ErrorAction SilentlyContinue)
            enabled_extension_ids = $enabled
            backup_path = $backupPath
            backup_sha256 = $backupHash
            manifest_path = $manifestPath
        }
    }
}
