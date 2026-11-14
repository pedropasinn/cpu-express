$ErrorActionPreference = 'Stop'
$project = Split-Path -Parent $PSScriptRoot
Push-Location $project
try {
    $exe = Join-Path $project 'target\release\cpu-express.exe'
    $run = Join-Path $project ('evidence\current\visual-refresh\run-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Force $run | Out-Null
    & "$PSScriptRoot\smoke-current.ps1" > (Join-Path $run 'smoke-console.log')
    $smokeResult = Get-Content -Raw (Join-Path $project 'evidence\current\native-smoke-result.json') | ConvertFrom-Json
    if ($smokeResult.exit_code -ne 0 -or -not $smokeResult.taskbar_surface_verified -or -not $smokeResult.independent_panel_verified) { throw 'Native smoke did not validate the current panel' }
    Remove-Item Env:CPU_EXPRESS_CAPTURE_DIR -ErrorAction SilentlyContinue
    Remove-Item Env:CPU_EXPRESS_OPEN_PANEL -ErrorAction SilentlyContinue
    function Invoke-Companion([string]$name, [string]$json, [int]$seconds, [bool]$demo) {
        $config = Join-Path $run ($name + '.json')
        $diagnostics = Join-Path $run ($name + '.jsonl')
        [IO.File]::WriteAllText($config, $json)
        $arguments = @('--config',('"'+$config+'"'),'--diagnostics',('"'+$diagnostics+'"'),'--smoke-seconds',"$seconds")
        if ($demo) { $arguments += '--demo' }
        $process = Start-Process -FilePath $exe -ArgumentList $arguments -WorkingDirectory $project -PassThru
        if (-not $process.WaitForExit(40000)) { Stop-Process -Id $process.Id; throw "$name timeout" }
        if ($process.ExitCode -ne 0) { throw "$name failed: $($process.ExitCode)" }
        Write-Output "$name exit 0"
    }
    Invoke-Companion 'after' '{"taskbar_mode":false,"floating_rect":[1100,100,640,180],"fps":60}' 12 $true
    Invoke-Companion 'real' '{"taskbar_mode":true,"fps":60}' 5 $false
    $env:CPU_EXPRESS_LAYOUT_TEST = '1'
    $env:CPU_EXPRESS_OPEN_PANEL = '1'
    $env:CPU_EXPRESS_CAPTURE_DIR = $run
    Invoke-Companion 'transitions' '{"taskbar_mode":true,"fps":60}' 20 $true
    python "$PSScriptRoot\validate-visual-refresh.py" $run
    if ($LASTEXITCODE -ne 0) { throw 'Evidence validation failed' }
} finally {
    Remove-Item Env:CPU_EXPRESS_LAYOUT_TEST -ErrorAction SilentlyContinue
    Remove-Item Env:CPU_EXPRESS_OPEN_PANEL -ErrorAction SilentlyContinue
    Remove-Item Env:CPU_EXPRESS_CAPTURE_DIR -ErrorAction SilentlyContinue
    Pop-Location
}
