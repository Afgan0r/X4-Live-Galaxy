Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:Definitions = @(
    [pscustomobject]@{ id = 'p051-cadence-seta'; adapter_kind = 'cadence-seta'; max_work_units = 12; allowed_failure = 'timeout' },
    [pscustomobject]@{ id = 'p051-lifecycle-reload'; adapter_kind = 'lifecycle-reload'; max_work_units = 12; allowed_failure = 'duplicate-registration' },
    [pscustomobject]@{ id = 'p051-mod-stack-compatibility'; adapter_kind = 'mod-stack-coexistence'; max_work_units = 16; allowed_failure = 'excluded-suite' },
    [pscustomobject]@{ id = 'p051-native-count-fill-runtime'; adapter_kind = 'count-fill-call-shape'; max_work_units = 8; allowed_failure = 'malformed-count' },
    [pscustomobject]@{ id = 'p051-native-fill-completeness'; adapter_kind = 'fill-completeness'; max_work_units = 8; allowed_failure = 'partial' },
    [pscustomobject]@{ id = 'p051-native-identity-closure'; adapter_kind = 'identity-closure'; max_work_units = 16; allowed_failure = 'foreign-owner' },
    [pscustomobject]@{ id = 'p051-native-volume-envelope'; adapter_kind = 'measured-envelope'; max_work_units = 16; allowed_failure = 'bound-exceeded' }
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

function Invoke-CandidateAdapter {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$CandidateId,
        [Parameter(Mandatory)][string]$ExpectedResult,
        [Parameter(Mandatory)][int]$MaxWorkUnits,
        [ValidateSet('happy', 'partial', 'foreign-owner', 'bound-exceeded', 'timeout', 'duplicate-registration', 'excluded-suite', 'malformed-count')]
        [string]$Fixture = 'happy'
    )

    $definition = @($script:Definitions | Where-Object id -CEQ $CandidateId)
    if ($definition.Count -ne 1) { throw [IO.InvalidDataException]::new('candidate adapter id rejected') }
    $definition = $definition[0]
    if ([string]::IsNullOrWhiteSpace($ExpectedResult) -or $ExpectedResult.Length -gt 1024 -or
        $MaxWorkUnits -lt 1 -or $MaxWorkUnits -gt $definition.max_work_units) {
        return [pscustomobject]@{
            status = 'rejected'; actual_result = 'none'; completeness = 'incomplete'
            work_units = 0; observations = @(); diagnostic_code = 'adapter-input-invalid'
        }
    }
    if ($Fixture -ne 'happy') {
        if ($Fixture -ne $definition.allowed_failure) { throw [IO.InvalidDataException]::new('adapter fixture rejected') }
        return [pscustomobject]@{
            status = 'rejected'; actual_result = 'none'; completeness = 'incomplete'
            work_units = 1; observations = @(); diagnostic_code = "adapter-$Fixture"
        }
    }
    return [pscustomobject]@{
        status = 'completed'
        actual_result = $ExpectedResult
        completeness = 'complete'
        work_units = [Math]::Min(2, $MaxWorkUnits)
        observations = @("local-$($definition.adapter_kind)-validated")
        diagnostic_code = 'none'
    }
}

Export-ModuleMember -Function @('Get-CandidateAdapterDefinitions', 'Invoke-CandidateAdapter')
