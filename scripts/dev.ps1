param([Parameter(ValueFromRemainingArguments=$true)][string[]]$CargoArgs)
$ErrorActionPreference = 'Stop'
$project = Split-Path -Parent $PSScriptRoot
$localTools = [IO.Path]::GetFullPath((Join-Path $project '..\..\.tools'))
if (Test-Path -LiteralPath (Join-Path $localTools 'cargo\bin\cargo.exe')) {
    $env:CARGO_HOME = Join-Path $localTools 'cargo'
    $env:RUSTUP_HOME = Join-Path $localTools 'rustup'
    $env:PATH = "$localTools\mingw64\bin;$localTools\cargo\bin;$env:PATH"
}
Push-Location $project
try {
    if (-not $CargoArgs) { $CargoArgs = @('run','--locked') }
    & cargo @CargoArgs
    $result = $LASTEXITCODE
} finally { Pop-Location }
exit $result
