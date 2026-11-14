$ErrorActionPreference = 'Stop'
$project = Split-Path -Parent $PSScriptRoot
$run = Join-Path $project ('evidence\current\weather\run-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Force $run | Out-Null
$config = Join-Path $run 'config.json'
$diagnostics = Join-Path $run 'weather.jsonl'
[IO.File]::WriteAllText($config, '{"taskbar_mode":false,"floating_rect":[100,140,800,240],"fps":60}')
$env:CPU_EXPRESS_LIVE_WEATHER = '1'
$env:CPU_EXPRESS_CAPTURE_DIR = $run
$env:CPU_EXPRESS_OPEN_PANEL = '1'
try {
    $exe = Join-Path $project 'target\release\cpu-express.exe'
    $arguments = @('--config',('"'+$config+'"'),'--diagnostics',('"'+$diagnostics+'"'),'--smoke-seconds','25')
    $process = Start-Process -FilePath $exe -ArgumentList $arguments -WorkingDirectory $project -PassThru
    if (-not $process.WaitForExit(55000)) { Stop-Process -Id $process.Id; throw 'Weather validation timeout' }
    if ($process.ExitCode -ne 0) { throw 'Weather validation failed' }
    python "$PSScriptRoot\validate-weather.py" $run
    if ($LASTEXITCODE -ne 0) { throw 'Live weather not validated' }
} finally {
    Remove-Item Env:CPU_EXPRESS_LIVE_WEATHER -ErrorAction SilentlyContinue
    Remove-Item Env:CPU_EXPRESS_CAPTURE_DIR -ErrorAction SilentlyContinue
    Remove-Item Env:CPU_EXPRESS_OPEN_PANEL -ErrorAction SilentlyContinue
}
