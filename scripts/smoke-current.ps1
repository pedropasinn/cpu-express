param([string]$Binary = 'target\rc3\release\cpu-express.exe')
$ErrorActionPreference = 'Stop'
$project = Split-Path -Parent $PSScriptRoot
$binaryPath = if ([IO.Path]::IsPathRooted($Binary)) { $Binary } else { [IO.Path]::GetFullPath((Join-Path $project $Binary)) }
$evidence = Join-Path $project 'evidence\rc3'
$run = Join-Path $project ('target\native-rc3-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Force $run | Out-Null
$config = Join-Path $run 'smoke.json'
$diagnostics = Join-Path $run 'native-smoke.jsonl'
[IO.File]::WriteAllText($config, '{"taskbar_mode":true,"always_on_top":true}')
$env:CPU_EXPRESS_CAPTURE_DIR = $run
$env:CPU_EXPRESS_OPEN_PANEL = '1'
$env:CPU_EXPRESS_OFFLINE = '1'
$arguments = @('--demo','--config',('"' + $config + '"'),'--smoke-seconds','12','--diagnostics',('"' + $diagnostics + '"'))
# This is the visible target app being validated, not a background helper.
$process = Start-Process -FilePath $binaryPath -ArgumentList $arguments -WorkingDirectory $project -WindowStyle Hidden -PassThru
$finished = $process.WaitForExit(45000)
if (-not $finished) { throw "Timeout no processo isolado $($process.Id); nenhuma instância foi encerrada à força." }
$exitCode = if ($finished) { $process.ExitCode } else { -1 }
$records = if (Test-Path -LiteralPath $diagnostics) { @(Get-Content -LiteralPath $diagnostics | ForEach-Object { $_ | ConvertFrom-Json }) } else { @() }
$last = $records | Select-Object -Last 1
$embedded = @($records | Where-Object { $_.native_window.taskbar_aligned -eq $true -and $_.native_window.height -eq $_.native_window.taskbar_height }).Count -gt 0
$panel = @($records | Where-Object { $_.native_window.panel.visible -eq $true -and $_.native_window.panel.height -gt 300 }).Count -gt 0
$fallback = @($records | Where-Object { $_.requested_taskbar_mode -eq $false -and $_.native_window.client_width -gt 0 }).Count -gt 0
$result = [ordered]@{
    binary=$binaryPath; binary_sha256=(Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash.ToLowerInvariant()
    exit_code=$exitCode; timed_out=(-not $finished); records=$records.Count
    taskbar_surface_verified=$embedded
    floating_fallback_verified=$fallback
    visual_confirmation='Não executada; somente consultas Win32 e framebuffer'
    independent_panel_verified=$panel
    strip_capture=(Test-Path -LiteralPath (Join-Path $run 'faixa.ppm'))
    panel_capture=(Test-Path -LiteralPath (Join-Path $run 'painel.ppm'))
    final_event=$last.event; rendered_frames=$last.rendered_frames
    native_window=$last.native_window; evidence_directory=$run
}
$result | ConvertTo-Json -Depth 5 | Set-Content -Encoding UTF8 (Join-Path $evidence 'native-smoke-result.json')
Copy-Item -LiteralPath $run -Destination (Join-Path $evidence (Split-Path $run -Leaf)) -Recurse
$result | ConvertTo-Json -Depth 5
if ($exitCode -ne 0 -or $last.event -ne 'shutdown' -or (-not $embedded -and -not $fallback) -or -not $result.strip_capture -or -not $panel) { exit 1 }
