[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][ValidateNotNullOrEmpty()][string]$DossierPath,
    [Parameter(Mandatory = $true)][ValidatePattern('^[a-z0-9][a-z0-9._-]{0,127}$')][string]$FindingId,
    [Parameter(Mandatory = $true)][ValidatePattern('^[a-z0-9][a-z0-9._-]{0,127}$')][string]$OwnerDecisionId,
    [Parameter(Mandatory = $true)][ValidateLength(1, 256)][string]$Rationale,
    [Parameter(Mandatory = $true)][ValidateLength(1, 256)][string]$RemainingRisk,
    [Parameter(Mandatory = $true)][ValidateNotNullOrEmpty()][string]$ExpiresAt,
    [Parameter(Mandatory = $true)][ValidateNotNullOrEmpty()][string]$OutputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'local-attestation.psm1') -Force

function Write-Failure([string]$Status) {
    [pscustomobject][ordered]@{
        schema_version = 'x4-owner-override-signing-result.v1'
        status = $Status
        authority_ready = $false
        artifact_written = $false
    } | ConvertTo-Json -Compress
    exit 1
}

try {
    if (-not (Test-Path -LiteralPath $DossierPath -PathType Leaf)) { Write-Failure 'OWNER_OVERRIDE_DOSSIER_MISSING' }
    $item = Get-Item -LiteralPath $DossierPath -Force
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or $item.Length -gt 65536) {
        Write-Failure 'OWNER_OVERRIDE_DOSSIER_INVALID'
    }
    [byte[]]$dossierBytes = [IO.File]::ReadAllBytes($item.FullName)
    $dossier = [Text.Encoding]::UTF8.GetString($dossierBytes) | ConvertFrom-Json
    if ($dossier.PSObject.Properties.Name -notcontains 'dossier_id' -or
        $dossier.PSObject.Properties.Name -notcontains 'findings') {
        Write-Failure 'OWNER_OVERRIDE_DOSSIER_INVALID'
    }
    $matching = @($dossier.findings | Where-Object { $_.id -eq $FindingId -and $_.disposition -eq 'known-failure' })
    if ($matching.Count -ne 1) { Write-Failure 'OVERRIDE_SCOPE_MISMATCH' }
    try {
        $expiry = [DateTimeOffset]::ParseExact($ExpiresAt, 'yyyy-MM-ddTHH:mm:ssZ', [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::AssumeUniversal)
    }
    catch { Write-Failure 'INVALID_FIELD_VALUE' }
    $issued = [DateTimeOffset]::UtcNow
    if ($expiry -le $issued -or $expiry -gt $issued.AddDays(90)) { Write-Failure 'OVERRIDE_EXPIRY_OUT_OF_RANGE' }

    $payload = [pscustomobject][ordered]@{
        schema_version = 'x4-owner-override.v1'
        override_id = "owner-override-$(([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($OwnerDecisionId)))).ToLowerInvariant().Substring(0, 32))"
        authority_purpose = 'owner-override'
        delegation_certificate_id = 'owner-override-delegation-v1'
        dossier_id = [string]$dossier.dossier_id
        dossier_digest = ([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($dossierBytes))).ToLowerInvariant()
        finding_id = $FindingId
        owner_decision_id = $OwnerDecisionId
        decision = 'accept-risk'
        rationale = $Rationale
        remaining_risk = $RemainingRisk
        issued_at = $issued.ToString('yyyy-MM-ddTHH:mm:ssZ')
        expires_at = $expiry.ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
        nonce = [guid]::NewGuid().ToString('N')
        signature_algorithm = 'ECDSA_P256_SHA256'
    }
    $signed = New-ProductionOwnerOverrideEnvelope -Payload $payload
    if ($signed.PSObject.Properties.Name -contains 'status') { Write-Failure ([string]$signed.status) }

    $fullOutput = [IO.Path]::GetFullPath($OutputPath)
    if (Test-Path -LiteralPath $fullOutput) { Write-Failure 'OWNER_OVERRIDE_OUTPUT_EXISTS' }
    $parent = Split-Path -Parent $fullOutput
    if (-not (Test-Path -LiteralPath $parent -PathType Container)) { Write-Failure 'OWNER_OVERRIDE_OUTPUT_PARENT_MISSING' }
    $parentItem = Get-Item -LiteralPath $parent -Force
    if (($parentItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { Write-Failure 'OWNER_OVERRIDE_OUTPUT_PATH_REJECTED' }
    $temporary = Join-Path $parent (".{0}.tmp" -f [guid]::NewGuid().ToString('N'))
    try {
        $json = $signed | ConvertTo-Json -Depth 8
        [IO.File]::WriteAllText($temporary, $json, [Text.UTF8Encoding]::new($false))
        Move-Item -LiteralPath $temporary -Destination $fullOutput
    }
    finally {
        if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary -Force }
    }
    [pscustomobject][ordered]@{
        schema_version = 'x4-owner-override-signing-result.v1'
        status = 'OWNER_OVERRIDE_SIGNED'
        authority_ready = $true
        artifact_written = $true
        override_id = $signed.override_id
        finding_id = $signed.finding_id
        payload_digest = $signed.payload_digest
    } | ConvertTo-Json -Compress
    exit 0
}
catch {
    Write-Failure 'OWNER_OVERRIDE_SIGNING_FAILED'
}
