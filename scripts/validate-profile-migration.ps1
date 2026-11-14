param([Parameter(Mandatory=$true)][string]$SourceConfig)
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$source=[IO.Path]::GetFullPath($SourceConfig)
$journey=Join-Path ([IO.Path]::GetDirectoryName($source)) ([IO.Path]::GetFileNameWithoutExtension($source)+'.journey.json')
$before=Get-Content -LiteralPath $journey -Raw -Encoding UTF8 | ConvertFrom-Json
$previousJourney=if ($before.version -eq 1) {$before} else {$before.journey}
$originalHash=(Get-FileHash -LiteralPath $journey -Algorithm SHA256).Hash
$run=Join-Path $project ('target\profile-migration-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Force $run | Out-Null
$isolated=Join-Path $run 'config.json'
$config=Get-Content -LiteralPath $source -Raw -Encoding UTF8 | ConvertFrom-Json
$config.taskbar_mode=$false
$config.floating_rect=@(1100,100,640,180)
$config | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $isolated -Encoding UTF8
$copyJourney=Join-Path $run 'config.journey.json'
Copy-Item -LiteralPath $journey -Destination $copyJourney
$exe=Join-Path $project 'target\release\cpu-express.exe'
$process=Start-Process -FilePath $exe -ArgumentList @('--config',('"'+$isolated+'"'),'--diagnostics',('"'+(Join-Path $run 'migration.jsonl')+'"')) -WorkingDirectory $project -WindowStyle Hidden -PassThru
Start-Sleep -Seconds 14
$process.Refresh()
if ($process.HasExited) {throw 'Isolated migration app exited early.'}
if (-not $process.CloseMainWindow()) {throw 'Could not request graceful isolated shutdown.'}
if (-not $process.WaitForExit(10000)) {throw 'Isolated migration shutdown timed out; inspect process.'}
if ($process.ExitCode -ne 0) {throw 'Migration app failed.'}
$after=Get-Content -LiteralPath $copyJourney -Raw -Encoding UTF8 | ConvertFrom-Json
if ($after.version -ne 3 -or $after.journey.distance -lt $previousJourney.distance -or $after.journey.active_seconds -lt $previousJourney.active_seconds) {throw 'Migration failed or lost route progress.'}
if ((Get-FileHash -LiteralPath $journey -Algorithm SHA256).Hash -ne $originalHash) {throw 'Source profile changed during isolated validation.'}
$backupPath=Join-Path $run ("config.journey.v"+$before.version+".backup.json")
if (-not (Test-Path -LiteralPath $backupPath) -or (Get-FileHash -LiteralPath $backupPath).Hash -ne $originalHash) {throw 'Exact migration backup missing.'}
if ($before.version -eq 2) {
  foreach ($painting in $before.service.owned_rewards) { if ($painting -notin $after.service.owned_rewards) {throw 'Owned painting lost.'} }
  if ($after.service.passengers_delivered -lt $before.service.passengers_delivered -or $after.service.cargo_delivered -lt $before.service.cargo_delivered) {throw 'Service totals lost.'}
}
$result=@{exit_code=0;scope='Isolated copy of historical profile; source untouched';binary_sha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLower();before_version=$before.version;before_distance=$previousJourney.distance;after_distance=$after.journey.distance;after_version=$after.version;owned_paintings=$after.service.owned_rewards;migration_backup_verified=$true}
$result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $project 'evidence\current\profile-migration-result.json') -Encoding UTF8
$result | ConvertTo-Json -Depth 8
