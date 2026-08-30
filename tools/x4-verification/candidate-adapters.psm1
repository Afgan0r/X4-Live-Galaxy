Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:Definitions = @(
    [pscustomobject]@{ id = 'p051-cadence-seta'; adapter_kind = 'cadence-seta'; max_work_units = 12 },
    [pscustomobject]@{ id = 'p051-lifecycle-reload'; adapter_kind = 'lifecycle-reload'; max_work_units = 12 },
    [pscustomobject]@{ id = 'p051-mod-stack-compatibility'; adapter_kind = 'mod-stack-coexistence'; max_work_units = 16 },
    [pscustomobject]@{ id = 'p051-native-count-fill-runtime'; adapter_kind = 'count-fill-call-shape'; max_work_units = 8 },
    [pscustomobject]@{ id = 'p051-native-fill-completeness'; adapter_kind = 'fill-completeness'; max_work_units = 8 },
    [pscustomobject]@{ id = 'p051-native-identity-closure'; adapter_kind = 'identity-closure'; max_work_units = 16 },
    [pscustomobject]@{ id = 'p051-native-volume-envelope'; adapter_kind = 'measured-envelope'; max_work_units = 16 }
)

function Get-CandidateAdapterDefinitions {
    foreach ($definition in @($script:Definitions | Sort-Object id)) {
        [pscustomobject]@{
            id = $definition.id
            adapter_kind = $definition.adapter_kind
            max_work_units = $definition.max_work_units
            classification = 'authenticated-local-contract'
            native_execution = 'forbidden'
        }
    }
}

function Test-ExactFields($Value, [string[]]$Expected) {
    if ($null -eq $Value -or $Value -isnot [pscustomobject]) { return $false }
    return (@($Value.PSObject.Properties.Name | Sort-Object) -join '|') -eq
        (@($Expected | Sort-Object) -join '|')
}

function Test-BoundedIdentity($Value) {
    return $Value -is [string] -and $Value -cmatch '^[a-z0-9][a-z0-9._:-]{0,63}$'
}

function Test-Integer($Value) {
    return $Value -is [byte] -or $Value -is [sbyte] -or
        $Value -is [int16] -or $Value -is [uint16] -or
        $Value -is [int32] -or $Value -is [uint32] -or
        $Value -is [int64] -or $Value -is [uint64]
}

function Test-BoundedIdentityArray($Value, [int]$ExpectedCount) {
    if ($Value -isnot [Array] -or $Value.Count -ne $ExpectedCount) { return $false }
    foreach ($element in $Value) {
        if (-not (Test-BoundedIdentity $element)) { return $false }
    }
    return $true
}

function New-Rejection([string]$Code) {
    return [pscustomobject]@{
        status = 'rejected'; actual_result = 'none'; completeness = 'incomplete'
        work_units = 1; observations = @(); diagnostic_code = $Code
    }
}

function New-Completion(
    [string]$Result,
    [string[]]$Observations,
    [int]$WorkUnits,
    [int]$MaxWorkUnits
) {
    if ($WorkUnits -gt $MaxWorkUnits) {
        return New-Rejection 'adapter-work-budget-exceeded'
    }
    return [pscustomobject]@{
        status = 'completed'; actual_result = $Result; completeness = 'complete'
        work_units = $WorkUnits; observations = @($Observations); diagnostic_code = 'none'
    }
}

function Invoke-CandidateAdapter {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$CandidateId,
        [Parameter(Mandatory)]$Fixture,
        [Parameter(Mandatory)][int]$MaxWorkUnits
    )

    $definition = @($script:Definitions | Where-Object id -CEQ $CandidateId)
    if ($definition.Count -ne 1) { throw [IO.InvalidDataException]::new('candidate adapter id rejected') }
    $definition = $definition[0]
    if ($MaxWorkUnits -lt 1 -or $MaxWorkUnits -gt $definition.max_work_units) {
        return New-Rejection 'adapter-input-invalid'
    }

    switch ($CandidateId) {
        'p051-cadence-seta' {
            if (-not (Test-ExactFields $Fixture @('real_ms', 'game_ms', 'seta_active', 'sample_count')) -or
                -not (Test-Integer $Fixture.real_ms) -or
                -not (Test-Integer $Fixture.game_ms) -or
                $Fixture.seta_active -isnot [bool] -or
                -not (Test-Integer $Fixture.sample_count) -or
                $Fixture.real_ms -ne 1000 -or $Fixture.game_ms -ne 6000 -or
                $Fixture.seta_active -ne $true -or $Fixture.sample_count -ne 4) {
                return New-Rejection 'adapter-cadence-semantic-invalid'
            }
            return New-Completion 'cadence:seta-ratio=6' @('samples=4', 'seta=true') 4 $MaxWorkUnits
        }
        'p051-lifecycle-reload' {
            if (-not (Test-ExactFields $Fixture @('registration_ids', 'reload_count')) -or
                -not (Test-BoundedIdentityArray $Fixture.registration_ids 1) -or
                -not (Test-Integer $Fixture.reload_count) -or
                $Fixture.reload_count -ne 1 -or
                @($Fixture.registration_ids | Sort-Object -Unique).Count -ne 1) {
                return New-Rejection 'adapter-lifecycle-semantic-invalid'
            }
            return New-Completion 'lifecycle:single-registration' @('reloads=1', 'registrations=1') 3 $MaxWorkUnits
        }
        'p051-mod-stack-compatibility' {
            if (-not (Test-ExactFields $Fixture @('enabled_mod_ids', 'excluded_mod_ids')) -or
                -not (Test-BoundedIdentityArray $Fixture.enabled_mod_ids 3) -or
                -not (Test-BoundedIdentityArray $Fixture.excluded_mod_ids 1)) {
                return New-Rejection 'adapter-mod-stack-semantic-invalid'
            }
            $enabled = @($Fixture.enabled_mod_ids | Sort-Object)
            $excluded = @($Fixture.excluded_mod_ids | Sort-Object)
            if (($enabled -join '|') -ne 'add-more-sectors|kuda-ai-tweaks|more-ai-economy-ships' -or
                ($excluded -join '|') -ne 'faction-enhancer') {
                return New-Rejection 'adapter-mod-stack-semantic-invalid'
            }
            return New-Completion 'mod-stack:declared-coexistence' @('enabled=3', 'excluded=1') 4 $MaxWorkUnits
        }
        'p051-native-count-fill-runtime' {
            if (-not (Test-ExactFields $Fixture @('reported_count', 'records')) -or
                -not (Test-Integer $Fixture.reported_count) -or
                -not (Test-BoundedIdentityArray $Fixture.records 3) -or
                $Fixture.reported_count -ne 3) {
                return New-Rejection 'adapter-count-fill-semantic-invalid'
            }
            return New-Completion 'count-fill:3-of-3' @('reported=3', 'filled=3') 3 $MaxWorkUnits
        }
        'p051-native-fill-completeness' {
            if (-not (Test-ExactFields $Fixture @('requested_count', 'returned_count', 'records')) -or
                -not (Test-Integer $Fixture.requested_count) -or
                -not (Test-Integer $Fixture.returned_count) -or
                -not (Test-BoundedIdentityArray $Fixture.records 3) -or
                $Fixture.requested_count -ne 3 -or $Fixture.returned_count -ne 3) {
                return New-Rejection 'adapter-fill-incomplete'
            }
            return New-Completion 'fill:complete=3' @('requested=3', 'returned=3') 3 $MaxWorkUnits
        }
        'p051-native-identity-closure' {
            if (-not (Test-ExactFields $Fixture @('native_id', 'canonical_id', 'owner_id', 'canonical_owner_id')) -or
                -not (Test-BoundedIdentity $Fixture.native_id) -or
                -not (Test-BoundedIdentity $Fixture.canonical_id) -or
                -not (Test-BoundedIdentity $Fixture.owner_id) -or
                -not (Test-BoundedIdentity $Fixture.canonical_owner_id) -or
                $Fixture.native_id -cne $Fixture.canonical_id -or
                $Fixture.owner_id -cne $Fixture.canonical_owner_id) {
                return New-Rejection 'adapter-identity-semantic-invalid'
            }
            $identityWorkUnits = @(
                $Fixture.native_id, $Fixture.canonical_id,
                $Fixture.owner_id, $Fixture.canonical_owner_id
            ).Count
            return New-Completion `
                "identity:object=$($Fixture.native_id)/owner=$($Fixture.owner_id)" `
                @("object=$($Fixture.native_id)", "owner=$($Fixture.owner_id)") `
                $identityWorkUnits `
                $MaxWorkUnits
        }
        'p051-native-volume-envelope' {
            if (-not (Test-ExactFields $Fixture @('sample_count', 'max_samples', 'payload_bytes', 'max_payload_bytes')) -or
                -not (Test-Integer $Fixture.sample_count) -or
                -not (Test-Integer $Fixture.max_samples) -or
                -not (Test-Integer $Fixture.payload_bytes) -or
                -not (Test-Integer $Fixture.max_payload_bytes) -or
                $Fixture.sample_count -lt 1 -or $Fixture.sample_count -gt $Fixture.max_samples -or
                $Fixture.max_samples -ne 16 -or $Fixture.payload_bytes -gt $Fixture.max_payload_bytes -or
                $Fixture.payload_bytes -lt 1 -or
                $Fixture.max_payload_bytes -ne 4096) {
                return New-Rejection 'adapter-volume-bound-exceeded'
            }
            $volumeWorkUnits = [int]$Fixture.sample_count
            return New-Completion `
                "volume:$($Fixture.sample_count)-samples/$($Fixture.payload_bytes)-bytes" `
                @(
                    "samples=$($Fixture.sample_count)/$($Fixture.max_samples)",
                    "bytes=$($Fixture.payload_bytes)/$($Fixture.max_payload_bytes)"
                ) `
                $volumeWorkUnits `
                $MaxWorkUnits
        }
    }
}

Export-ModuleMember -Function @('Get-CandidateAdapterDefinitions', 'Invoke-CandidateAdapter')
