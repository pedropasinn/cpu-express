#Requires -Version 5.1
<#
.SYNOPSIS
Measure the real CPU Express process without generating artificial CPU load.
.DESCRIPTION
Seconds is the total requested application run time. Only samples whose complete
interval follows WarmupSeconds enter the summary. The application
closes itself through --smoke-seconds. Ctrl+C also cleans up only this run's PID.
An isolated 30 FPS config preserves the user's normal preferences. OutputDir is
a parent folder; every run creates a new timestamped child folder.
Keep the train visible for a visible-window baseline. Use a separate run for a
minimized comparison, and record that action when interpreting the report.
.EXAMPLE
.\scripts\measure-windows.ps1 -Exe .\cpu-express.exe -Seconds 300
.EXAMPLE
.\scripts\measure-windows.ps1 -Exe .\cpu-express.exe -Seconds 900 -IncludeDiagnostics
.NOTES
Requires Windows and PowerShell 5.1+. Installs nothing and sends no data.
WorkingSet64 includes shared and private resident pages. PrivateMemorySize64
is the process Private Bytes counter, not another measure of physical RAM.
https://learn.microsoft.com/dotnet/api/system.diagnostics.process.workingset64
https://learn.microsoft.com/dotnet/api/system.diagnostics.process.privatememorysize64
#>
[CmdletBinding()]
param(
    [string]$Exe,
    [ValidateRange(5, 86400)][int]$Seconds = 300,
    [ValidateRange(0, 86397)][int]$WarmupSeconds = 15,
    [string]$OutputDir,
    [switch]$IncludeDiagnostics
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'Run this measurement script on Windows.'
}
if ($WarmupSeconds -gt ($Seconds - 3)) {
    throw 'Leave at least three seconds after WarmupSeconds.'
}
if ([string]::IsNullOrWhiteSpace($Exe)) {
    $candidates = @(
        (Join-Path $PSScriptRoot '..\cpu-express.exe'),
        (Join-Path $PSScriptRoot '..\target\x86_64-pc-windows-msvc\release\cpu-express.exe'),
        (Join-Path $PSScriptRoot '..\target\x86_64-pc-windows-gnu\release\cpu-express.exe'),
        (Join-Path $PSScriptRoot '..\target\release\cpu-express.exe')
    )
    $Exe = $candidates | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1
    if ([string]::IsNullOrWhiteSpace($Exe)) { throw 'Specify -Exe with the path to cpu-express.exe.' }
}
$resolvedExe = (Resolve-Path -LiteralPath $Exe).ProviderPath
if (-not (Test-Path -LiteralPath $resolvedExe -PathType Leaf)) {
    throw 'Exe must identify the built CPU Express executable.'
}
if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $PSScriptRoot '..\measurements'
}
$outputRoot = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputDir)
[void][IO.Directory]::CreateDirectory($outputRoot)
$runName = 'cpu-express-{0}-{1}' -f (Get-Date -Format 'yyyyMMdd-HHmmss'), ([guid]::NewGuid().ToString('N').Substring(0, 8))
$runDirectory = Join-Path $outputRoot $runName
[void][IO.Directory]::CreateDirectory($runDirectory)
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
$configPath = Join-Path $runDirectory 'benchmark-config.json'
$csvPath = Join-Path $runDirectory 'process-samples.csv'
$summaryPath = Join-Path $runDirectory 'summary.json'
$diagnosticsPath = Join-Path $runDirectory 'application.jsonl'
$benchmarkConfig = [ordered]@{
    min_speed_kmh = 12.0; max_speed_kmh = 100.0; smoothing_k = 1.8
    fps = 30; metrics_interval_ms = 1000; show_metrics = $true; always_on_top = $true
}
[IO.File]::WriteAllText($configPath, ($benchmarkConfig | ConvertTo-Json), $utf8NoBom)

function Get-Median([double[]]$Values) {
    if ($null -eq $Values -or $Values.Count -eq 0) { return $null }
    $sorted = @($Values | Sort-Object)
    $middle = [int][Math]::Floor($sorted.Count / 2)
    if (($sorted.Count % 2) -eq 1) { return [double]$sorted[$middle] }
    return ([double]$sorted[$middle - 1] + [double]$sorted[$middle]) / 2.0
}

function Get-Distribution([object[]]$Rows, [string]$Property) {
    if ($null -eq $Rows -or $Rows.Count -eq 0) { return $null }
    $values = @($Rows | ForEach-Object { [double]($_.$Property) })
    $stats = $values | Measure-Object -Minimum -Maximum -Average
    return [ordered]@{
        minimum = $stats.Minimum; median = (Get-Median $values)
        maximum = $stats.Maximum; mean = $stats.Average
        first = $values[0]; last = $values[$values.Count - 1]
        last_minus_first = $values[$values.Count - 1] - $values[0]
    }
}

function Get-MemorySlope([object[]]$Rows, [string]$Property) {
    if ($null -eq $Rows -or $Rows.Count -lt 30) { return $null }
    $span = $Rows[$Rows.Count - 1].elapsed_seconds - $Rows[0].elapsed_seconds
    if ($span -lt 60.0) { return $null }
    $xMean = ($Rows | Measure-Object -Property elapsed_seconds -Average).Average
    $yMean = ($Rows | Measure-Object -Property $Property -Average).Average
    $numerator = 0.0
    $denominator = 0.0
    foreach ($row in $Rows) {
        $dx = [double]$row.elapsed_seconds - $xMean
        $numerator += $dx * ([double]($row.$Property) - $yMean)
        $denominator += $dx * $dx
    }
    if ($denominator -le 0.0) { return $null }
    return ($numerator / $denominator) * 60.0 / 1048576.0
}

# Windows file names cannot contain double quotes. These are file paths ending
# in .json/.jsonl, so quoting does not leave a trailing backslash before a quote.
function Quote-FileArgument([string]$Value) {
    if ($Value.Contains('"')) { throw 'A file argument cannot contain a quote.' }
    return '"' + $Value + '"'
}

$metadataWarnings = New-Object 'System.Collections.Generic.List[string]'
$logicalProcessors = [Environment]::ProcessorCount
$physicalCores = $null
$processorModels = @()
$osInfo = [ordered]@{ version = [Environment]::OSVersion.VersionString }
try {
    $processors = @(Get-CimInstance -ClassName Win32_Processor)
    $count = ($processors | Measure-Object -Property NumberOfLogicalProcessors -Sum).Sum
    if ($count -gt 0) { $logicalProcessors = [int]$count }
    $physicalCores = ($processors | Measure-Object -Property NumberOfCores -Sum).Sum
    $processorModels = @($processors | ForEach-Object { $_.Name.Trim() })
    $windowsInfo = Get-CimInstance -ClassName Win32_OperatingSystem
    $osInfo = [ordered]@{
        caption = $windowsInfo.Caption; version = $windowsInfo.Version
        build = $windowsInfo.BuildNumber; architecture = $windowsInfo.OSArchitecture
        total_visible_memory_bytes = [double]$windowsInfo.TotalVisibleMemorySize * 1024.0
    }
} catch {
    $metadataWarnings.Add('Some optional hardware/OS metadata could not be read: ' + $_.Exception.Message)
}
$exeInfo = Get-Item -LiteralPath $resolvedExe
$exeVersion = [Diagnostics.FileVersionInfo]::GetVersionInfo($resolvedExe)
$exeSha256 = (Get-FileHash -LiteralPath $resolvedExe -Algorithm SHA256).Hash.ToLowerInvariant()
$argumentLine = '--smoke-seconds {0} --config {1}' -f $Seconds, (Quote-FileArgument $configPath)
if ($IncludeDiagnostics) {
    $argumentLine += ' --diagnostics ' + (Quote-FileArgument $diagnosticsPath)
}

$samples = New-Object 'System.Collections.Generic.List[object]'
$launchedProcess = $null
$launchedStartTicks = $null
$launchedId = $null
$exitCode = $null
$forcedStop = $false
$runError = $null
$startedUtc = [DateTime]::UtcNow
$stopwatch = [Diagnostics.Stopwatch]::StartNew()
Write-Host "Measuring $resolvedExe for $Seconds seconds; no artificial load."
Write-Host "Reports: $runDirectory"
try {
    $launchedProcess = Start-Process -FilePath $resolvedExe -ArgumentList $argumentLine -PassThru
    $launchedId = $launchedProcess.Id
    $launchedStartTicks = $launchedProcess.StartTime.ToUniversalTime().Ticks
    $launchedProcess.Refresh()
    $previousWall = $stopwatch.Elapsed.TotalSeconds
    $previousCpu = $launchedProcess.TotalProcessorTime.TotalSeconds
    while (-not $launchedProcess.HasExited) {
        Start-Sleep -Milliseconds 1000
        $launchedProcess.Refresh()
        if ($launchedProcess.HasExited) { break }
        $wall = $stopwatch.Elapsed.TotalSeconds
        if ($wall -gt ($Seconds + 30)) {
            throw 'The application did not close within its requested duration plus 30 seconds.'
        }
        $processorSeconds = $launchedProcess.TotalProcessorTime.TotalSeconds
        $elapsed = $wall - $previousWall
        $cpuDelta = [Math]::Max(0.0, $processorSeconds - $previousCpu)
        $oneCorePercent = 100.0 * $cpuDelta / $elapsed
        $samples.Add([pscustomobject][ordered]@{
            elapsed_seconds = $wall
            interval_seconds = $elapsed
            process_cpu_seconds = $processorSeconds
            interval_cpu_seconds = $cpuDelta
            cpu_percent_one_core = $oneCorePercent
            cpu_percent_machine = $oneCorePercent / $logicalProcessors
            working_set_bytes = $launchedProcess.WorkingSet64
            private_memory_bytes = $launchedProcess.PrivateMemorySize64
            post_warmup = ($previousWall -ge $WarmupSeconds)
        })
        $previousWall = $wall
        $previousCpu = $processorSeconds
    }
    $launchedProcess.WaitForExit()
    $exitCode = $launchedProcess.ExitCode
    if ($exitCode -ne 0) { throw "The application exited with code $exitCode." }
    if ($stopwatch.Elapsed.TotalSeconds -lt ($Seconds - 1)) {
        throw 'The application exited before the requested measurement duration.'
    }
} catch {
    $runError = $_.Exception.Message
    Write-Warning $runError
} finally {
    # Never search or kill by executable name. Verify the exact process start
    # time as well as PID before closing anything, even if a PID was recycled.
    if ($null -ne $launchedProcess) {
        try {
            $launchedProcess.Refresh()
            if (-not $launchedProcess.HasExited -and $null -eq $runError) {
                $runError = 'Measurement interrupted before the application completed.'
            }
            if (-not $launchedProcess.HasExited -and $null -eq $launchedStartTicks) {
                $launchedId = $launchedProcess.Id
                $launchedStartTicks = $launchedProcess.StartTime.ToUniversalTime().Ticks
            }
            if (-not $launchedProcess.HasExited -and $null -ne $launchedStartTicks) {
                $current = Get-Process -Id $launchedId -ErrorAction SilentlyContinue
                if ($null -ne $current -and $current.StartTime.ToUniversalTime().Ticks -eq $launchedStartTicks) {
                    [void]$launchedProcess.CloseMainWindow()
                    if (-not $launchedProcess.WaitForExit(3000)) {
                        $launchedProcess.Kill()
                        [void]$launchedProcess.WaitForExit(3000)
                        $forcedStop = $true
                    }
                }
                if ($null -ne $current) { $current.Dispose() }
            }
            if ($launchedProcess.HasExited) { $exitCode = $launchedProcess.ExitCode }
        } catch {
            $metadataWarnings.Add('Could not complete cleanup of this run: ' + $_.Exception.Message)
        }
    }
    $stopwatch.Stop()
    if ($samples.Count -gt 0) {
        $samples | Export-Csv -LiteralPath $csvPath -NoTypeInformation -Encoding UTF8
    } else {
        $csvHeader = 'elapsed_seconds,interval_seconds,process_cpu_seconds,interval_cpu_seconds,cpu_percent_one_core,cpu_percent_machine,working_set_bytes,private_memory_bytes,post_warmup'
        [IO.File]::WriteAllText($csvPath, $csvHeader + [Environment]::NewLine, $utf8NoBom)
    }
    $steady = @($samples | Where-Object { $_.post_warmup })
    $weightedCpu = $null
    if ($steady.Count -gt 0) {
        $observedSeconds = ($steady | Measure-Object -Property interval_seconds -Sum).Sum
        $observedCpu = ($steady | Measure-Object -Property interval_cpu_seconds -Sum).Sum
        $weightedCpu = 100.0 * $observedCpu / $observedSeconds
    }
    $summary = [ordered]@{
        started_utc = $startedUtc.ToString('o')
        requested_seconds = $Seconds; actual_wall_seconds = $stopwatch.Elapsed.TotalSeconds
        warmup_seconds = $WarmupSeconds; sample_interval_target_seconds = 1
        samples_total = $samples.Count; samples_after_warmup = $steady.Count
        process_id = $launchedId; exit_code = $exitCode; forced_stop = $forcedStop
        error = $runError; metadata_warnings = @($metadataWarnings.ToArray())
        executable = [ordered]@{
            path = $resolvedExe; bytes = $exeInfo.Length; sha256 = $exeSha256
            file_version = $exeVersion.FileVersion; product_version = $exeVersion.ProductVersion
        }
        machine = [ordered]@{
            os = $osInfo; logical_processors = $logicalProcessors
            physical_cores = $physicalCores; cpu_models = $processorModels
            powershell_version = $PSVersionTable.PSVersion.ToString()
        }
        configuration = $benchmarkConfig
        diagnostics_enabled = [bool]$IncludeDiagnostics
        cpu_percent_one_core = (Get-Distribution $steady 'cpu_percent_one_core')
        cpu_percent_machine = (Get-Distribution $steady 'cpu_percent_machine')
        cpu_percent_one_core_weighted_mean = $weightedCpu
        cpu_percent_machine_weighted_mean = $(if ($null -ne $weightedCpu) { $weightedCpu / $logicalProcessors } else { $null })
        working_set_bytes = (Get-Distribution $steady 'working_set_bytes')
        private_memory_bytes = (Get-Distribution $steady 'private_memory_bytes')
        working_set_slope_mib_per_minute = (Get-MemorySlope $steady 'working_set_bytes')
        private_memory_slope_mib_per_minute = (Get-MemorySlope $steady 'private_memory_bytes')
        interpretation = @(
            'CPU one-core percentage can exceed 100; machine percentage divides by logical processor count.',
            'Working set includes shared pages; private bytes are private committed memory, not physical RAM.',
            'Memory slopes require at least 30 samples spanning 60 seconds; a trend alone does not establish a leak.',
            'These measurements describe this run and machine. The observer itself contributes to total machine CPU.',
            'Normal user configuration was not read or overwritten. No synthetic CPU load was generated.',
            'Empty executable version fields mean no version resource was embedded; identify the build by SHA-256.'
        )
    }
    [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 8), $utf8NoBom)
    if ($null -ne $launchedProcess) { $launchedProcess.Dispose() }
    Write-Host "Saved: $summaryPath"
    Write-Host "Saved: $csvPath"
}
if ($null -ne $runError) { throw "Measurement was incomplete: $runError See $summaryPath" }

