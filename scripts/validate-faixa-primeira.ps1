param([string]$Binary = 'target\debug\cpu-express.exe')
$ErrorActionPreference = 'Stop'
$project = Split-Path -Parent $PSScriptRoot
$source = [IO.Path]::GetFullPath((Join-Path $project $Binary))
$run = Join-Path $project ('target\codex-native-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $run | Out-Null
# Copiar evita bloquear o executável que Cargo precisa atualizar. Não altera evidence/current.
$exe = Join-Path $run 'cpu-express.exe'
Copy-Item -LiteralPath $source -Destination $exe
$hash = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash
$environment = @{}
foreach ($key in @('CPU_EXPRESS_CAPTURE_DIR','CPU_EXPRESS_LAYOUT_TEST','CPU_EXPRESS_OPEN_PANEL')) {
    $environment[$key] = [Environment]::GetEnvironmentVariable($key, 'Process')
}
$results = @()
try {
    foreach ($scenario in @('primeira', 'alternancias')) {
        $dir = Join-Path $run $scenario
        New-Item -ItemType Directory -Path $dir | Out-Null
        $config = Join-Path $dir 'config.json'
        $diagnostics = Join-Path $dir 'native.jsonl'
        $env:CPU_EXPRESS_CAPTURE_DIR = $dir
        $env:CPU_EXPRESS_OPEN_PANEL = '1'
        $env:CPU_EXPRESS_LAYOUT_TEST = $null
        $seconds = 12
        $arguments = @('--config', ('"'+$config+'"'), '--diagnostics', ('"'+$diagnostics+'"'))
        if ($scenario -eq 'alternancias') {
            [IO.File]::WriteAllText($config, '{"taskbar_mode":true,"taskbar_rounded":true,"taskbar_fit_height":false,"taskbar_height":120}')
            $env:CPU_EXPRESS_LAYOUT_TEST = '1'
            $seconds = 20
            $arguments += '--demo'
        }
        $arguments += @('--smoke-seconds', "$seconds")
        # Aplicativo sob teste; sem tocar no perfil normal ou em outra instância.
        $process = Start-Process -FilePath $exe -ArgumentList $arguments -WorkingDirectory $project -PassThru
        $finished = $process.WaitForExit(($seconds + 15) * 1000)
        if (-not $finished) { throw "O processo de teste $($process.Id) não encerrou; não foi terminado à força." }
        $records = @(Get-Content -LiteralPath $diagnostics -Encoding UTF8 | ForEach-Object { $_ | ConvertFrom-Json })
        $bars = @($records | Where-Object { $_.native_window.taskbar_width -gt 0 })
        $results += [ordered]@{
            scenario = $scenario; exit_code = $process.ExitCode; records = $records.Count
            final_event = $records[-1].event
            taskbar_available = ($bars.Count -gt 0)
            floating_observed = (@($records | Where-Object { $_.requested_taskbar_mode -eq $false -and $_.native_window.client_width -gt 0 }).Count -gt 0)
            panel_observed = (@($records | Where-Object { $_.native_window.panel.visible -eq $true }).Count -gt 0)
            config_exists = (Test-Path -LiteralPath $config)
            visual_confirmation = 'NÃO EXECUTADO: captura do framebuffer e consultas nativas não certificam compositor nem cliques'
        }
        if ($process.ExitCode -ne 0) { throw "Falha no cenário $scenario : $($process.ExitCode)" }
    }
} finally {
    foreach ($key in $environment.Keys) { [Environment]::SetEnvironmentVariable($key, $environment[$key], 'Process') }
    [ordered]@{ binary = $exe; sha256 = $hash; scenarios = $results; directory = $run } |
        ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $run 'result.json') -Encoding UTF8
}
Get-Content -LiteralPath (Join-Path $run 'result.json') -Encoding UTF8
