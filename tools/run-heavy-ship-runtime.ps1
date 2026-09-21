[CmdletBinding()]
param(
    [ValidateSet('Normal', 'Seta')][string]$Mode = 'Normal',
    [int]$X4ProcessId,
    [string]$DebugLog,
    [string]$PresentMonPath,
    [string]$BridgePath,
    [string]$LimitsFile,
    [string]$EvidenceRoot,
    [ValidateRange(10, 3600)][int]$CaptureSeconds = 65,
    [ValidateRange(10, 3600)][int]$SetaWaitSeconds = 300,
    [switch]$SelfTest
)
$ErrorActionPreference = 'Stop'

function Get-GameTimes([string]$Text) {
    @([regex]::Matches($Text, '(?m)^\[Economy_Verbose\]\s+(?<time>\d+(?:\.\d+)?)') |
        ForEach-Object { [double]$_.Groups['time'].Value })
}

function Get-RuntimeDiagnostics([string]$Text) {
    @([regex]::Matches($Text,
        'Live Galaxy Carrier B: event=(?<event>[a-z_]+) detail=(?<detail>[a-z_]+)') |
        ForEach-Object { "$($_.Groups['event'].Value):$($_.Groups['detail'].Value)" })
}

function Get-RuntimeDiagnosticFailures([string[]]$Diagnostics) {
    @($Diagnostics | Where-Object {
        $_ -notin @(
            'transition:sampled',
            'transition:core_changed',
            'initialized:initialized'
        )
    })
}

function Read-GameTimes([string]$Path, [long]$Offset) {
    $stream = [System.IO.File]::Open($Path, 'Open', 'Read', 'ReadWrite')
    try {
        if ($stream.Length -lt $Offset) { $Offset = 0 }
        [void]$stream.Seek($Offset, 'Begin')
        $reader = [System.IO.StreamReader]::new($stream, [Text.Encoding]::UTF8, $true, 4096, $true)
        try { $text = $reader.ReadToEnd() } finally { $reader.Dispose() }
        [pscustomobject]@{
            Offset = $stream.Length
            Times = @(Get-GameTimes $text)
            Diagnostics = @(Get-RuntimeDiagnostics $text)
        }
    } finally { $stream.Dispose() }
}

function Measure-GameFactor([System.Collections.IList]$Samples) {
    if ($Samples.Count -lt 2) { return $null }
    $wall = ($Samples[-1].Wall - $Samples[0].Wall).TotalSeconds
    $game = [double]$Samples[-1].Game - [double]$Samples[0].Game
    if ($wall -le 0 -or $game -lt 0) { return $null }
    $game / $wall
}

function Add-GameSample([System.Collections.IList]$Samples, $Read) {
    if ($Read.Times.Count -gt 0) {
        [void]$Samples.Add([pscustomobject]@{ Wall = [DateTime]::UtcNow; Game = $Read.Times[-1] })
    }
}

function Limit-GameSampleWindow([System.Collections.IList]$Samples, [double]$Seconds) {
    while ($Samples.Count -gt 2) {
        if (($Samples[-1].Wall - $Samples[0].Wall).TotalSeconds -le $Seconds) { break }
        $Samples.RemoveAt(0)
    }
}

function Start-OwnedProcess(
    [string]$FilePath,
    [string[]]$Arguments,
    [string]$StandardOutputPath,
    [string]$StandardErrorPath,
    [string]$WorkingDirectory
) {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $FilePath
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    if (-not [string]::IsNullOrWhiteSpace($WorkingDirectory)) {
        $info.WorkingDirectory = $WorkingDirectory
    }
    foreach ($argument in $Arguments) { [void]$info.ArgumentList.Add($argument) }
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $info
    [void]$process.Start()
    [pscustomobject]@{
        Process = $process
        StandardOutput = $process.StandardOutput.ReadToEndAsync()
        StandardError = $process.StandardError.ReadToEndAsync()
        StandardOutputPath = $StandardOutputPath
        StandardErrorPath = $StandardErrorPath
    }
}

function Complete-OwnedProcess($Owned, [switch]$Stop) {
    if ($null -eq $Owned) { return }
    if ($Stop -and -not $Owned.Process.HasExited) { $Owned.Process.Kill() }
    $Owned.Process.WaitForExit()
    [IO.File]::WriteAllText($Owned.StandardOutputPath,
        $Owned.StandardOutput.GetAwaiter().GetResult(), [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($Owned.StandardErrorPath,
        $Owned.StandardError.GetAwaiter().GetResult(), [Text.UTF8Encoding]::new($false))
}

function Invoke-SelfTest {
    $times = @(Get-GameTimes "ignored`n[Economy_Verbose] 10.25 sample`n[Economy_Verbose] 72.25 sample")
    if ($times.Count -ne 2 -or $times[0] -ne 10.25 -or $times[1] -ne 72.25) {
        throw 'HEAVY_RUNTIME_GAME_CLOCK_PARSE'
    }
    $diagnosticText = "Live Galaxy Carrier B: event=transition detail=sampled`n" +
        'Live Galaxy Carrier B: event=transition detail=core_changed stage=revalidate'
    $diagnostics = @(Get-RuntimeDiagnostics $diagnosticText)
    if ($diagnostics.Count -ne 2 -or $diagnostics[0] -ne 'transition:sampled' `
        -or $diagnostics[1] -ne 'transition:core_changed') {
        throw 'HEAVY_RUNTIME_DIAGNOSTIC_PARSE'
    }
    if (@(Get-RuntimeDiagnosticFailures $diagnostics).Count -ne 0) {
        throw 'HEAVY_RUNTIME_DIAGNOSTIC_VERDICT'
    }
    $failures = @(Get-RuntimeDiagnosticFailures @($diagnostics + 'transition:unexpected'))
    if ($failures.Count -ne 1 -or $failures[0] -ne 'transition:unexpected') {
        throw 'HEAVY_RUNTIME_DIAGNOSTIC_FAILURE'
    }
    $samples = [Collections.ArrayList]::new()
    [void]$samples.Add([pscustomobject]@{ Wall = [DateTime]'2026-01-01T00:00:00Z'; Game = 10.0 })
    [void]$samples.Add([pscustomobject]@{ Wall = [DateTime]'2026-01-01T00:00:10Z'; Game = 72.0 })
    if ([Math]::Abs((Measure-GameFactor $samples) - 6.2) -gt 0.001) {
        throw 'HEAVY_RUNTIME_SETA_FACTOR'
    }
    $samples[-1] = [pscustomobject]@{ Wall = [DateTime]'2026-01-01T00:00:10Z'; Game = 20.0 }
    if ((Measure-GameFactor $samples) -ge 4.0) { throw 'HEAVY_RUNTIME_NORMAL_AS_SETA' }
    [void]$samples.Insert(0, [pscustomobject]@{ Wall = [DateTime]'2025-12-31T23:59:40Z'; Game = 0.0 })
    Limit-GameSampleWindow $samples 15
    if ($samples.Count -ne 2 -or (Measure-GameFactor $samples) -ne 1.0) {
        throw 'HEAVY_RUNTIME_SETA_WINDOW'
    }
    $out = [IO.Path]::GetTempFileName(); $err = [IO.Path]::GetTempFileName()
    $script = [IO.Path]::Combine([IO.Path]::GetTempPath(), "$([Guid]::NewGuid()).ps1")
    try {
        $pwsh = (Get-Process -Id $PID).Path
        [IO.File]::WriteAllText($script,
            '[Console]::Out.Write($args[0] + "|" + [Environment]::CurrentDirectory)',
            [Text.UTF8Encoding]::new($false))
        $owned = Start-OwnedProcess -FilePath $pwsh -Arguments @('-NoProfile', '-File',
            $script, 'value with spaces') `
            -StandardOutputPath $out -StandardErrorPath $err -WorkingDirectory ([IO.Path]::GetTempPath())
        Complete-OwnedProcess $owned
        $actualOut = Get-Content -LiteralPath $out -Raw
        $actualErr = Get-Content -LiteralPath $err -Raw
        $expectedOut = "value with spaces|$([IO.Path]::GetTempPath().TrimEnd('\'))"
        if ($actualOut.TrimEnd('\') -ne $expectedOut -or -not [string]::IsNullOrEmpty($actualErr)) {
            throw "HEAVY_RUNTIME_ARGUMENT_PRESERVATION:out=<$actualOut>:err=<$actualErr>"
        }
    } finally {
        Remove-Item -LiteralPath $out, $err, $script -Force -ErrorAction SilentlyContinue
    }
    Write-Output 'heavy ship runtime runner self-test passed'
}

if ($SelfTest) { Invoke-SelfTest; exit 0 }
foreach ($required in @($DebugLog, $PresentMonPath, $BridgePath, $LimitsFile, $EvidenceRoot)) {
    if ([string]::IsNullOrWhiteSpace($required)) { throw 'HEAVY_RUNTIME_ARGUMENT_MISSING' }
}
if (-not (Get-Process -Id $X4ProcessId -ErrorAction SilentlyContinue)) { throw 'HEAVY_RUNTIME_X4_ABSENT' }
foreach ($path in @($DebugLog, $PresentMonPath, $BridgePath, $LimitsFile)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "HEAVY_RUNTIME_FILE_MISSING:$path" }
}

$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$captureId = "055-fps-loaded-$($Mode.ToLowerInvariant())-$stamp"
$runId = "055-gate-a-$($Mode.ToLowerInvariant())-$stamp"
$captureRoot = Join-Path $EvidenceRoot "frame-captures\$captureId"
$runRoot = Join-Path $EvidenceRoot "runs\$runId"
$dataRoot = Join-Path $runRoot 'data'
New-Item -ItemType Directory -Force -Path $captureRoot, $dataRoot | Out-Null
$statusPath = Join-Path $runRoot 'runner-status.json'
$resultPath = Join-Path $runRoot 'runner-result.json'
$offset = (Get-Item -LiteralPath $DebugLog).Length
trap {
    if (-not [string]::IsNullOrWhiteSpace($statusPath)) {
        @{ state = 'failed'; run_id = $runId; reason = $_.Exception.Message } | ConvertTo-Json |
            Set-Content -LiteralPath $statusPath -Encoding utf8NoBOM
    }
    $host.UI.WriteErrorLine($_.ToString())
    break
}

if ($Mode -eq 'Seta') {
    @{ state = 'waiting-for-seta'; run_id = $runId } | ConvertTo-Json | Set-Content -LiteralPath $statusPath -Encoding utf8NoBOM
    $waitSamples = [Collections.ArrayList]::new()
    $deadline = [DateTime]::UtcNow.AddSeconds($SetaWaitSeconds)
    do {
        Start-Sleep -Seconds 1
        $read = Read-GameTimes $DebugLog $offset; $offset = $read.Offset
        Add-GameSample $waitSamples $read
        Limit-GameSampleWindow $waitSamples 15
        $factor = Measure-GameFactor $waitSamples
        $observedFor = if ($waitSamples.Count -gt 1) {
            ($waitSamples[-1].Wall - $waitSamples[0].Wall).TotalSeconds
        } else { 0 }
        if ($observedFor -ge 5 -and $null -ne $factor -and $factor -ge 4.0 -and $factor -le 8.0) { break }
    } while ([DateTime]::UtcNow -lt $deadline -and (Get-Process -Id $X4ProcessId -ErrorAction SilentlyContinue))
    if ($null -eq $factor -or $factor -lt 4.0 -or $factor -gt 8.0) { throw 'HEAVY_RUNTIME_SETA_UNPROVEN' }
}

$csv = Join-Path $captureRoot "x4-loaded-$($Mode.ToLowerInvariant()).csv"
$pmOut = Join-Path $captureRoot 'presentmon.stdout.log'; $pmErr = Join-Path $captureRoot 'presentmon.stderr.log'
$bridgeOut = Join-Path $runRoot 'bridge.stdout.log'; $bridgeErr = Join-Path $runRoot 'bridge.stderr.log'
$useFrameViewSdk = [IO.Path]::GetFileName($PresentMonPath) -ieq 'FvSDKTestClient_Public.exe'
$pmArgs = if ($useFrameViewSdk) { @('--test_case', '6') } else {
    @('--process_id', [string]$X4ProcessId, '--timed', [string]$CaptureSeconds,
        '--terminate_after_timed', '--output_file', $csv,
        '--session_name', "LiveGalaxy-$Mode-$stamp", '--no_console_stats')
}
$pm = $null; $bridge = $null; $failure = $null
try {
    $bridgeArgs = @('--data-dir', $dataRoot, '--limits-file', $LimitsFile, '--ship-faction', 'argon')
    $bridge = Start-OwnedProcess -FilePath $BridgePath -Arguments $bridgeArgs `
        -StandardOutputPath $bridgeOut -StandardErrorPath $bridgeErr
    $started = [DateTime]::UtcNow; $offset = (Get-Item -LiteralPath $DebugLog).Length
    $runSamples = [Collections.ArrayList]::new()
    $runtimeDiagnostics = [Collections.ArrayList]::new()
    $segment = 0
    do {
        $segment += 1
        $segmentOut = if ($useFrameViewSdk) {
            Join-Path $captureRoot "frameview-$segment.stdout.log"
        } else { $pmOut }
        $segmentErr = if ($useFrameViewSdk) {
            Join-Path $captureRoot "frameview-$segment.stderr.log"
        } else { $pmErr }
        $pm = Start-OwnedProcess -FilePath $PresentMonPath -Arguments $pmArgs `
            -StandardOutputPath $segmentOut -StandardErrorPath $segmentErr `
            -WorkingDirectory $(if ($useFrameViewSdk) { $captureRoot } else { $null })
        @{ state = 'capturing'; run_id = $runId; bridge_pid = $bridge.Process.Id
            frame_capture_pid = $pm.Process.Id; frame_capture_segment = $segment } |
            ConvertTo-Json | Set-Content -LiteralPath $statusPath -Encoding utf8NoBOM
        while (-not $pm.Process.HasExited) {
            Start-Sleep -Seconds 1; $pm.Process.Refresh(); $bridge.Process.Refresh()
            $read = Read-GameTimes $DebugLog $offset; $offset = $read.Offset; Add-GameSample $runSamples $read
            foreach ($diagnostic in $read.Diagnostics) { [void]$runtimeDiagnostics.Add($diagnostic) }
            if ($bridge.Process.HasExited) { throw "HEAVY_RUNTIME_BRIDGE_EXITED:$($bridge.Process.ExitCode)" }
            if (([DateTime]::UtcNow - $started).TotalSeconds -gt $CaptureSeconds + 15) {
                throw 'HEAVY_RUNTIME_CAPTURE_TIMEOUT'
            }
        }
        Complete-OwnedProcess $pm
        if ($pm.Process.ExitCode -ne 0) { throw "HEAVY_RUNTIME_FRAME_CAPTURE_EXITED:$($pm.Process.ExitCode)" }
        $pm = $null
        if (-not $useFrameViewSdk) { break }
    } while (([DateTime]::UtcNow - $started).TotalSeconds -lt $CaptureSeconds)
    if ($useFrameViewSdk) {
        $sdkCsv = @(Get-ChildItem -LiteralPath $captureRoot -Filter 'FvSDKPerFrameStreamDataT*.csv' -File)
        if ($sdkCsv.Count -lt $segment) { throw 'HEAVY_RUNTIME_FRAME_CAPTURE_MISSING' }
    }
    $read = Read-GameTimes $DebugLog $offset
    Add-GameSample $runSamples $read
    foreach ($diagnostic in $read.Diagnostics) { [void]$runtimeDiagnostics.Add($diagnostic) }
} catch { $failure = $_ } finally {
    Complete-OwnedProcess $pm -Stop
    Complete-OwnedProcess $bridge -Stop
}
if ($failure) { throw $failure }

$csvFiles = if ($useFrameViewSdk) {
    @(Get-ChildItem -LiteralPath $captureRoot -Filter 'FvSDKPerFrameStreamDataT*.csv' -File |
        Sort-Object Name | ForEach-Object { $_.FullName })
} else { @($csv) }
$rows = @($csvFiles | ForEach-Object { Import-Csv -LiteralPath $_ } |
    Where-Object { $_.MsBetweenPresents -ne 'NA' -and
        (-not $useFrameViewSdk -or $_.ProcessId -eq [string]$X4ProcessId) })
$history = Join-Path $dataRoot 'operational-history.jsonl'
$events = @(Get-Content -LiteralPath $history | ForEach-Object { $_ | ConvertFrom-Json })
$commits = @($events | Where-Object { $_.state -eq 'committed' }).Count
$bad = @($events | Where-Object { $_.state -in @('failed', 'rejected') }).Count
$runtimeFailures = @(Get-RuntimeDiagnosticFailures $runtimeDiagnostics)
$coreChanged = @($runtimeDiagnostics | Where-Object { $_ -eq 'transition:core_changed' }).Count
$gameFactor = Measure-GameFactor $runSamples
$gameFactorSeconds = if ($runSamples.Count -gt 1) {
    ($runSamples[-1].Wall - $runSamples[0].Wall).TotalSeconds
} else { 0 }
$frameValues = @($rows | ForEach-Object { [double]$_.MsBetweenPresents })
$averageFrame = ($frameValues | Measure-Object -Average).Average
$captureErrors = if ($useFrameViewSdk) {
    @(Get-ChildItem -LiteralPath $captureRoot -Filter 'frameview-*.stderr.log' -File |
        ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw }) -join "`n"
} else { Get-Content -LiteralPath $pmErr -Raw }
$lostEvents = $captureErrors -match '(?i)lost\s+\d+\s+events'
$valid = $rows.Count -gt 0 -and $commits -gt 0 -and $bad -eq 0 `
    -and $runtimeFailures.Count -eq 0 -and -not $lostEvents
if ($Mode -eq 'Seta') {
    $valid = $valid -and $gameFactorSeconds -ge 10 -and $null -ne $gameFactor `
        -and $gameFactor -ge 4.0 -and $gameFactor -le 8.0
}
$result = [ordered]@{ status = $(if ($valid) { 'passed' } else { 'failed' }); mode = $Mode.ToLowerInvariant()
    run_id = $runId; capture_id = $captureId; x4_process_id = $X4ProcessId; frames = $rows.Count
    average_fps = $(if ($averageFrame -gt 0) { 1000 / $averageFrame } else { 0 }); commits = $commits
    rejected_or_failed = $bad; lost_presentmon_events = $lostEvents
    runtime_diagnostic_failures = $runtimeFailures.Count
    runtime_diagnostic_failure_details = @($runtimeFailures | Sort-Object -Unique)
    core_changed_transitions = $coreChanged
    game_time_factor = $gameFactor; game_time_sample_seconds = $gameFactorSeconds
    capture_seconds = ([DateTime]::UtcNow - $started).TotalSeconds
    frame_capture_backend = $(if ($useFrameViewSdk) { 'frameview-sdk' } else { 'presentmon' })
    frame_capture_segments = $csvFiles.Count }
$result | ConvertTo-Json | Set-Content -LiteralPath $resultPath -Encoding utf8NoBOM
$result | ConvertTo-Json | Set-Content -LiteralPath $statusPath -Encoding utf8NoBOM
$result | ConvertTo-Json -Compress
if (-not $valid) { throw 'HEAVY_RUNTIME_EVIDENCE_FAILED' }
