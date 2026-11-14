# Fixture sintética v3, sem ler ou alterar o perfil real do usuário.
. "$PSScriptRoot\rc3-common.ps1"
$destination = Join-Path $project ('target\fixture v3 '+[DateTime]::Now.ToString('yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $destination | Out-Null
$journey = Get-Content (Join-Path $project 'tests\fixtures\rc7\mid-segment.journey.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$service = [ordered]@{
    branch=[ordered]@{pending=$null;current=$null;discoveries=@()}
    composition=@('observation','sleeper','freight');passengers=@();automatic_contracts=$true;contract=$null
    last_processed_visit=1;passengers_boarded=0;passengers_delivered=0;cargo_delivered=0;contracts_completed=0
    credits=0;owned_rewards=@('aurora','ocean');collection=@();arrivals=@()
}
Write-Json (Join-Path $destination 'config.journey.json') ([ordered]@{version=3;journey=$journey;service=$service})
Write-Json (Join-Path $destination 'config.json') ([ordered]@{taskbar_mode=$false;floating_rect=@(100,100,640,180);start_paused=$true})
Write-Json (Join-Path $destination 'ORIGEM.json') ([ordered]@{
    scope='Fixture sintética v3, não é save do Pedro nem exportação de execução rc.2'
    journey_source='tests/fixtures/rc7/mid-segment.journey.json'
    service='Estado de migração vazio, uma chegada e pinturas Aurora/Ocean; formato v3 com despacho e ramal'
})
Copy-Item -LiteralPath $destination -Destination (Join-Path $evidence 'fixture-v3') -Recurse
Write-Output (Join-Path $destination 'config.json')
