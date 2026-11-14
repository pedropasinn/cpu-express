param([int]$Seconds = 120)
$ErrorActionPreference = 'Stop'
if ($Seconds -lt 30 -or $Seconds -gt 3600) { throw 'Use 30..3600 seconds.' }
$project = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $project 'target\release\cpu-express.exe'
$run = Join-Path $project ('evidence\current\soak\run-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Force $run | Out-Null
$all = @()
foreach ($panel in @($false,$true)) {
    $tag = if ($panel) { 'panel-open' } else { 'panel-closed' }
    $config = Join-Path $run ($tag+'.json')
    $diagnostics = Join-Path $run ($tag+'.jsonl')
    [IO.File]::WriteAllText($config,'{"taskbar_mode":false,"floating_rect":[1100,100,640,180],"fps":60,"quiet_mode":true}')
    $previousPanel = $env:CPU_EXPRESS_OPEN_PANEL
    try {
        if ($panel) { $env:CPU_EXPRESS_OPEN_PANEL = '1' } else { Remove-Item Env:CPU_EXPRESS_OPEN_PANEL -ErrorAction SilentlyContinue }
        $process = Start-Process -FilePath $exe -ArgumentList @('--config',('"'+$config+'"'),'--diagnostics',('"'+$diagnostics+'"'),'--smoke-seconds',"$Seconds") -WorkingDirectory $project -WindowStyle Hidden -PassThru
    } finally { if ($null -eq $previousPanel) {Remove-Item Env:CPU_EXPRESS_OPEN_PANEL -ErrorAction SilentlyContinue} else {$env:CPU_EXPRESS_OPEN_PANEL=$previousPanel} }
    $samples = @()
    $clock = [Diagnostics.Stopwatch]::StartNew()
    while (-not $process.HasExited) {
        if ($clock.Elapsed.TotalSeconds -gt ($Seconds+30)) { Stop-Process -Id $process.Id; throw 'Isolated soak process timed out.' }
        $process.Refresh()
        $samples += @{elapsed=$clock.Elapsed.TotalSeconds;cpu_seconds=$process.TotalProcessorTime.TotalSeconds;working_set=$process.WorkingSet64;private_bytes=$process.PrivateMemorySize64}
        Start-Sleep -Seconds 1
    }
    $process.WaitForExit()
    if ($process.ExitCode -ne 0) { throw "$tag exit $($process.ExitCode)" }
    $records = @(Get-Content -LiteralPath $diagnostics | ForEach-Object {$_ | ConvertFrom-Json})
    if ($records[-1].event -ne 'shutdown') { throw "$tag missing graceful shutdown" }
    $all += @{scenario=$tag;seconds=$clock.Elapsed.TotalSeconds;exit_code=$process.ExitCode;samples=$samples;last=$records[-1]}
}
$result = @{scope='Short native runs; process CPU/memory and application frame work, not compositor or battery';binary_sha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLower();runs=$all}
$result | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath (Join-Path $run 'result.json') -Encoding UTF8
Copy-Item -LiteralPath (Join-Path $run 'result.json') -Destination (Join-Path $project 'evidence\current\soak\result.json')
Write-Output "Validated $($all.Count) native scenarios in $run"
