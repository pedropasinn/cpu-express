param([int]$Jobs = 2)
$project = Split-Path -Parent $PSScriptRoot
$localTools = [IO.Path]::GetFullPath((Join-Path $project '..\..\.tools'))
if (Test-Path -LiteralPath (Join-Path $localTools 'cargo\bin\cargo.exe')) {
    $env:CARGO_HOME = Join-Path $localTools 'cargo'
    $env:RUSTUP_HOME = Join-Path $localTools 'rustup'
    $env:PATH = "$localTools\mingw64\bin;$localTools\cargo\bin;$env:PATH"
}
$evidence = Join-Path $project 'evidence\current'
New-Item -ItemType Directory -Force $evidence | Out-Null
$results = @()
Push-Location $project
try {
    $checks = @(
        @{ name='format'; arguments=@('fmt','--all','--','--check') },
        @{ name='clippy'; arguments=@('clippy','--locked','--all-targets','-j',"$Jobs",'--','-D','warnings') },
        @{ name='tests'; arguments=@('test','--locked','--all-targets','-j',"$Jobs") }
    )
    foreach ($check in $checks) {
        $started = Get-Date
        $arguments = $check.arguments
        & cargo @arguments *> (Join-Path $evidence ($check.name + '.log'))
        $code = $LASTEXITCODE
        $results += @{ name=$check.name; exit_code=$code; seconds=((Get-Date)-$started).TotalSeconds }
        $results | ConvertTo-Json | Set-Content -Encoding UTF8 (Join-Path $evidence 'checks.json')
        Write-Output ($check.name + ': exit ' + $code)
        if ($code -ne 0) { exit $code }
    }
} finally { Pop-Location }
