# Funções locais de integridade para a distribuição rc.3; nenhuma consulta de rede.
$ErrorActionPreference = 'Stop'
$project = Split-Path -Parent $PSScriptRoot
$root = [IO.Path]::GetFullPath((Join-Path $project '..\..'))
$evidence = Join-Path $project 'evidence\rc3'
$binary = Join-Path $project 'target\rc3\release\cpu-express.exe'
$utf8 = New-Object Text.UTF8Encoding($false)
function Write-Json($Path, $Value) {
    [IO.File]::WriteAllText($Path, ($Value | ConvertTo-Json -Depth 40), $utf8)
}
function Get-Digest($Path) { (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() }
function Get-SourceHashes {
    $result = [ordered]@{}
    $files = @('Cargo.toml', 'Cargo.lock')
    foreach ($directory in @('src','data','tests','examples','.cargo')) {
        $path = Join-Path $project $directory
        if (Test-Path $path) {
            $files += Get-ChildItem -LiteralPath $path -File -Recurse | ForEach-Object { $_.FullName.Substring($project.Length+1).Replace('\','/') }
        }
    }
    foreach ($file in ($files | Sort-Object)) { $result[$file] = Get-Digest (Join-Path $project $file) }
    return $result
}
function Assert-SourceHashes($Expected) {
    $current = Get-SourceHashes
    $names = @($Expected.PSObject.Properties.Name)
    if ($names.Count -ne $current.Count) { throw 'Lista de fontes mudou após validação.' }
    foreach ($name in $names) { if ($current[$name] -ne $Expected.$name) { throw "Fonte mudou: $name" } }
}
function Initialize-Toolchain {
    $localTools = Join-Path $root '.tools'
    $env:CARGO_HOME = Join-Path $localTools 'cargo'
    $env:RUSTUP_HOME = Join-Path $localTools 'rustup'
    $env:PATH = "$localTools\mingw64\bin;$localTools\cargo\bin;$env:PATH"
    $env:CARGO_BUILD_JOBS = '2'
    $env:CARGO_NET_OFFLINE = 'true'
    $env:CARGO_TARGET_DIR = Join-Path $project 'target\rc3'
    $env:TEMP = Join-Path $project 'target\rc3\tmp'
    $env:TMP = $env:TEMP
    New-Item -ItemType Directory -Force $env:TEMP | Out-Null
}
