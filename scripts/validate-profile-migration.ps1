param([Parameter(Mandatory=$true)][string]$SourceConfig,
      [string]$Binary = 'target\rc3\release\cpu-express.exe')
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$source=[IO.Path]::GetFullPath($SourceConfig)
$journey=Join-Path ([IO.Path]::GetDirectoryName($source)) ([IO.Path]::GetFileNameWithoutExtension($source)+'.journey.json')
$before=Get-Content -LiteralPath $journey -Raw -Encoding UTF8 | ConvertFrom-Json
if ($before.version -ne 3) { throw 'Este ensaio exige uma cópia de perfil v3 e valida v3 -> v4.' }
$previousJourney=if ($before.version -eq 1) {$before} else {$before.journey}
$originalHash=(Get-FileHash -LiteralPath $journey -Algorithm SHA256).Hash
$configHash=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
$run=Join-Path $project ('target\profile-migration-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Force $run | Out-Null
$isolated=Join-Path $run 'config.json'
$config=Get-Content -LiteralPath $source -Raw -Encoding UTF8 | ConvertFrom-Json
$config.taskbar_mode=$false
$config.floating_rect=@(1100,100,640,180)
$config | Add-Member -NotePropertyName start_paused -NotePropertyValue $true -Force
[IO.File]::WriteAllText($isolated, ($config | ConvertTo-Json -Depth 30), (New-Object Text.UTF8Encoding($false)))
$copyJourney=Join-Path $run 'config.journey.json'
Copy-Item -LiteralPath $journey -Destination $copyJourney
$exe=if ([IO.Path]::IsPathRooted($Binary)) { $Binary } else { Join-Path $project $Binary }
$offlineBefore=$env:CPU_EXPRESS_OFFLINE
$env:CPU_EXPRESS_OFFLINE='1'
try {
$process=Start-Process -FilePath $exe -ArgumentList @('--config',('"'+$isolated+'"'),'--diagnostics',('"'+(Join-Path $run 'migration.jsonl')+'"')) -WorkingDirectory $project -WindowStyle Hidden -PassThru
Start-Sleep -Seconds 14
$process.Refresh()
if ($process.HasExited) {throw 'Isolated migration app exited early.'}
if (-not $process.CloseMainWindow()) {throw 'Could not request graceful isolated shutdown.'}
if (-not $process.WaitForExit(10000)) {throw 'Isolated migration shutdown timed out; inspect process.'}
if ($process.ExitCode -ne 0) {throw 'Migration app failed.'}
} finally { $env:CPU_EXPRESS_OFFLINE=$offlineBefore }
$records=@(Get-Content (Join-Path $run 'migration.jsonl') -Encoding UTF8 | ForEach-Object { $_ | ConvertFrom-Json })
if ($records[-1].event -ne 'shutdown' -or @($records | Where-Object { $_.local_weather.enabled -ne $false }).Count -gt 0) { throw 'Encerramento ou bloqueio de rede não confirmado.' }
$after=Get-Content -LiteralPath $copyJourney -Raw -Encoding UTF8 | ConvertFrom-Json
if ($after.version -ne 4 -or $after.line -ne 'aurora-v1' -or $after.catalog_version -ne 1 -or $after.journey.distance -ne $previousJourney.distance -or $after.journey.active_seconds -ne $previousJourney.active_seconds) {throw 'Migration failed or changed paused route progress.'}
if ((Get-FileHash -LiteralPath $journey -Algorithm SHA256).Hash -ne $originalHash) {throw 'Source profile changed during isolated validation.'}
if ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -ne $configHash) {throw 'Source config changed during isolated validation.'}
$backupPath=Join-Path $run ("config.journey.v"+$before.version+".backup.json")
if (-not (Test-Path -LiteralPath $backupPath) -or (Get-FileHash -LiteralPath $backupPath).Hash -ne $originalHash) {throw 'Exact migration backup missing.'}
if ($before.version -eq 3) {
  foreach ($painting in $before.service.owned_rewards) { if ($painting -notin $after.service.owned_rewards) {throw 'Owned painting lost.'} }
  if ($after.service.passengers_delivered -lt $before.service.passengers_delivered -or $after.service.cargo_delivered -lt $before.service.cargo_delivered) {throw 'Service totals lost.'}
}
if (($before.service | ConvertTo-Json -Depth 40 -Compress) -ne ($after.service | ConvertTo-Json -Depth 40 -Compress)) { throw 'O serviço pausado mudou durante a migração.' }
$evidence=Join-Path $project 'evidence\rc3'
New-Item -ItemType Directory -Force $evidence | Out-Null
Copy-Item -LiteralPath $run -Destination (Join-Path $evidence (Split-Path $run -Leaf)) -Recurse
$result=@{exit_code=0;scope='Cópia isolada de fixture sintética v3; origem intocada; pausado; rede desativada';binary_sha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLower();before_version=$before.version;before_distance=$previousJourney.distance;after_distance=$after.journey.distance;after_version=$after.version;owned_paintings=$after.service.owned_rewards;migration_backup_verified=$true;service_preserved=$true;source_sha256=$originalHash.ToLower();directory=$run}
$result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $evidence 'profile-migration-result.json') -Encoding UTF8
$result | ConvertTo-Json -Depth 8
