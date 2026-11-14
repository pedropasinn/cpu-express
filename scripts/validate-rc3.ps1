# Executar depois de build-rc3.ps1. Não compila e não acessa rede/perfil real.
$ErrorActionPreference = 'Stop'
& powershell -NoProfile -ExecutionPolicy Bypass -File "$PSScriptRoot\package-rc3.ps1" -Preliminary
if ($LASTEXITCODE -ne 0) { throw 'Pacote preliminar falhou.' }
& powershell -NoProfile -ExecutionPolicy Bypass -File "$PSScriptRoot\validate-portable.ps1"
if ($LASTEXITCODE -ne 0) { throw 'Validação portátil falhou.' }
. "$PSScriptRoot\rc3-common.ps1"
$portable = Get-Content (Join-Path $evidence 'portable-result.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$source = & powershell -NoProfile -ExecutionPolicy Bypass -File "$PSScriptRoot\prepare-v3-fixture.ps1"
if ($LASTEXITCODE -ne 0) { throw 'Preparação da fixture falhou.' }
& powershell -NoProfile -ExecutionPolicy Bypass -File "$PSScriptRoot\validate-profile-migration.ps1" -SourceConfig $source -Binary $portable.executable
if ($LASTEXITCODE -ne 0) { throw 'Migração falhou.' }
& powershell -NoProfile -ExecutionPolicy Bypass -File "$PSScriptRoot\smoke-current.ps1" -Binary $portable.executable
if ($LASTEXITCODE -ne 0) { throw 'Smoke falhou.' }
