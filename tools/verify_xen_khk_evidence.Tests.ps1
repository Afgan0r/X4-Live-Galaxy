$scriptPath = Join-Path $PSScriptRoot 'verify_xen_khk_evidence.ps1'

function New-EvidenceFixture {
    param([string]$Json)

    $path = Join-Path $TestDrive 'evidence.md'
    @(
        '# Evidence fixture',
        '',
        '```json hostile-claim-register',
        $Json,
        '```'
    ) | Set-Content -LiteralPath $path -Encoding utf8
    return $path
}

function Get-ValidSkeleton {
    return @'
{
  "schema_version": "1.0",
  "source_boundary": "static X4 9.00 and repository contracts only",
  "scope": {
    "xen_primary_pressure": true,
    "khk_observed_when_present": true,
    "autonomous_hostile_minds": false,
    "government_institutions": false,
    "hostile_motives": false,
    "hostile_diplomacy": false,
    "hostile_architecture_selected": false,
    "hostile_write_primitives": false,
    "hostile_control_channels": false,
    "critical_path_dependency": false,
    "phase8_inventory_only": true
  },
  "coverage": {
    "requirements": ["RES-01", "RES-02", "RES-03"],
    "decisions": ["D-01", "D-02", "D-03", "D-04", "D-05", "D-06", "D-07", "D-08"]
  },
  "sources": [
    {"id":"x4-version-dat","kind":"installed_x4_file","path":"version.dat","boundary":"installed X4 9.00","allowed_conclusions":["installed_version"]},
    {"id":"x4-jobs","kind":"installed_x4_file","path":"08.cat::libraries/jobs.xml","boundary":"installed X4 9.00 catalog","allowed_conclusions":["xen_job_configuration"]},
    {"id":"x4-khaak-activity","kind":"installed_x4_file","path":"08.cat::md/khaak_activity.xml","boundary":"installed X4 9.00 catalog","allowed_conclusions":["khk_activity_configuration"]},
    {"id":"khaakfinder-precedent","kind":"installed_extension_precedent","path":"extensions/z_ram_khaakfinder","boundary":"installed extension v101","allowed_conclusions":["visibility_precedent"]},
    {"id":"project-contract","kind":"repository_contract","path":"AGENTS.md|.planning/PROJECT.md|.planning/REQUIREMENTS.md|.planning/ROADMAP.md|02-CONTEXT.md","boundary":"repository planning","allowed_conclusions":["observation_only_scope"]}
  ],
  "claims": [
    {"id":"XEN-STATE-01","faction":"XEN","area":"state","classification":"observed","source_id":"x4-jobs","permitted_conclusion":"xen_job_configuration","non_gating":true,"future_owner":"Phase 1","evidence_needed":"runtime probe"},
    {"id":"KHK-STATE-01","faction":"KHK","area":"state","classification":"observed","source_id":"x4-khaak-activity","permitted_conclusion":"khk_activity_configuration","non_gating":true,"future_owner":"Phase 1","evidence_needed":"runtime probe"}
  ]
}
'@
}

function Invoke-Verifier([string]$EvidencePath) {
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $scriptPath -EvidencePath $EvidencePath -Stage skeleton 2>$null | Out-Null
    return $LASTEXITCODE
}

function New-FullFixture([scriptblock]$Mutate) {
    $source = Join-Path $PSScriptRoot '..\.planning\phases\02-hostile-faction-research-track\02-XEN-KHK-EVIDENCE.md'
    $path = Join-Path $TestDrive 'full-evidence.md'
    $content = Get-Content -LiteralPath $source -Raw -Encoding utf8
    & $Mutate ([ref]$content)
    Set-Content -LiteralPath $path -Value $content -Encoding utf8
    return $path
}

function Invoke-FullVerifier([string]$EvidencePath) {
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $scriptPath -EvidencePath $EvidencePath -Stage full 2>$null | Out-Null
    return $LASTEXITCODE
}

Describe 'verify_xen_khk_evidence' {
    It 'accepts a valid skeleton without X4 runtime access' {
        $path = New-EvidenceFixture (Get-ValidSkeleton)
        (Invoke-Verifier $path) | Should Be 0
    }

    It 'rejects duplicate source identifiers' {
        $json = (Get-ValidSkeleton).Replace('"x4-version-dat","kind"', '"x4-jobs","kind"')
        (Invoke-Verifier (New-EvidenceFixture $json)) | Should Not Be 0
    }

    It 'rejects unknown claim source identifiers' {
        $json = (Get-ValidSkeleton).Replace('"source_id":"x4-jobs"', '"source_id":"unknown-source"')
        (Invoke-Verifier (New-EvidenceFixture $json)) | Should Not Be 0
    }

    It 'rejects unallowlisted source kind path or boundary' {
        $json = (Get-ValidSkeleton).Replace('"installed_x4_file"', '"network_source"')
        (Invoke-Verifier (New-EvidenceFixture $json)) | Should Not Be 0
    }

    It 'rejects a conclusion outside its source scope' {
        $json = (Get-ValidSkeleton).Replace('"permitted_conclusion":"xen_job_configuration"', '"permitted_conclusion":"khk_activity_configuration"')
        (Invoke-Verifier (New-EvidenceFixture $json)) | Should Not Be 0
    }
}

Describe 'verify_xen_khk_evidence full-stage invariants' {
    It 'accepts the valid full register' { (Invoke-FullVerifier (New-FullFixture { param($c) })) | Should Be 0 }
    It 'rejects missing faction coverage' { (Invoke-FullVerifier (New-FullFixture { param($c) $c.Value = $c.Value.Replace('"faction":"KHK"', '"faction":"OTHER"') })) | Should Not Be 0 }
    It 'rejects altered RES coverage' { (Invoke-FullVerifier (New-FullFixture { param($c) $c.Value = $c.Value.Replace('"RES-03"', '"RES-99"') })) | Should Not Be 0 }
    It 'rejects altered D coverage' { (Invoke-FullVerifier (New-FullFixture { param($c) $c.Value = $c.Value.Replace('"D-08"', '"D-99"') })) | Should Not Be 0 }
    It 'rejects each forbidden positive scope flag' {
        foreach ($flag in @('autonomous_hostile_minds','government_institutions','hostile_motives','hostile_diplomacy','hostile_architecture_selected','hostile_write_primitives','hostile_control_channels','critical_path_dependency')) {
            (Invoke-FullVerifier (New-FullFixture { param($c) $c.Value = $c.Value.Replace(('"' + $flag + '": false'), ('"' + $flag + '": true')) })) | Should Not Be 0
        }
    }
    It 'rejects duplicate claim IDs and invalid classification' {
        (Invoke-FullVerifier (New-FullFixture { param($c) $c.Value = $c.Value.Replace('"KHK-STATE-01"', '"XEN-STATE-01"') })) | Should Not Be 0
        (Invoke-FullVerifier (New-FullFixture { param($c) $c.Value = $c.Value.Replace('"classification":"observed"', '"classification":"invalid"') })) | Should Not Be 0
    }
    It 'rejects unknown claims missing each required deferral field' {
        foreach ($field in @('non_gating','future_owner','evidence_needed')) {
            (Invoke-FullVerifier (New-FullFixture {
                param($c)
                $c.Value = $c.Value.Replace(('"' + $field + '":true'), ('"' + $field + '":false'))
                $c.Value = $c.Value.Replace(('"' + $field + '":"Phase 1 and Phase 7 X4 validation"'), ('"' + $field + '":""'))
                $c.Value = $c.Value.Replace(('"' + $field + '":"attributable disposable X4 9.00 event export and independent readback"'), ('"' + $field + '":""'))
            })) | Should Not Be 0
        }
    }
    It 'rejects malformed and duplicate named fences' {
        (Invoke-FullVerifier (New-FullFixture { param($c) $c.Value = $c.Value.Replace('"schema_version"', '"schema_version" BROKEN') })) | Should Not Be 0
        (Invoke-FullVerifier (New-FullFixture { param($c) $c.Value += [Environment]::NewLine + '```json hostile-claim-register' + [Environment]::NewLine + '{}' + [Environment]::NewLine + '```' })) | Should Not Be 0
    }
}
