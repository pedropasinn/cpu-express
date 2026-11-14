param([switch]$IntegrityOnly)
. "$PSScriptRoot\rc3-common.ps1"
Add-Type -AssemblyName System.IO.Compression.FileSystem
Add-Type -AssemblyName System.IO.Compression
$name = 'CPU-Express-0.4.0-rc.3'
$run = Join-Path $project ('target\portable acceptance '+[DateTime]::Now.ToString('yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $run | Out-Null
foreach ($suffix in @('','-Fonte','-Evidencias')) {
    $zip = [IO.Compression.ZipFile]::OpenRead((Join-Path $root "release\$name$suffix.zip"))
    try {
        foreach ($entry in $zip.Entries) {
            $destination = [IO.Path]::GetFullPath((Join-Path $run $entry.FullName))
            if (-not $destination.StartsWith(($run+'\'),[StringComparison]::OrdinalIgnoreCase)) { throw 'Caminho inseguro no ZIP.' }
            [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($destination)) | Out-Null
            if ($entry.Name) { [IO.Compression.ZipFileExtensions]::ExtractToFile($entry,$destination,$false) }
        }
    } finally { $zip.Dispose() }
}
$portable = Join-Path $run $name
foreach ($line in (Get-Content (Join-Path $portable 'SHA256SUMS.txt') -Encoding UTF8)) {
    $parts = $line -split '  ',2
    if ((Get-Digest (Join-Path $portable $parts[1])) -ne $parts[0]) { throw "Payload corrompido: $($parts[1])" }
}
foreach ($pair in @(@('cpu-express','SOURCE-MANIFEST.json'),@('evidence\rc3','EVIDENCE-MANIFEST.json'))) {
    $directory = Join-Path $run $pair[0]
    $manifest = Get-Content (Join-Path $directory $pair[1]) -Raw -Encoding UTF8 | ConvertFrom-Json
    foreach ($entry in $manifest.PSObject.Properties) {
        if ((Get-Digest (Join-Path $directory $entry.Name)) -ne $entry.Value) { throw "Manifesto divergente: $($entry.Name)" }
    }
    if (@(Get-ChildItem $directory -Recurse -File).Count -ne (@($manifest.PSObject.Properties).Count+1)) { throw 'Arquivos extras ou ausentes no manifesto.' }
}
$exe = Join-Path $portable 'cpu-express.exe'
if ((Get-Digest $exe) -ne (Get-Digest $binary)) { throw 'Executável extraído diverge.' }
$info = Get-Content (Join-Path $portable 'BUILD-INFO.json') -Raw -Encoding UTF8 | ConvertFrom-Json
if ($info.executable_sha256 -ne (Get-Digest $exe)) { throw 'BUILD-INFO diverge.' }
if (-not $IntegrityOnly) {
    $config = Join-Path $run 'acceptance.json'
    [IO.File]::WriteAllText($config,'{"taskbar_mode":false,"floating_rect":[100,100,640,180],"fps":60}', $utf8)
    $diagnostics = Join-Path $run 'acceptance.jsonl'
    $oldPath = $env:PATH
    $saved = @{}
    Get-ChildItem Env:CPU_EXPRESS_* | ForEach-Object { $saved[$_.Name]=$_.Value; [Environment]::SetEnvironmentVariable($_.Name,$null,'Process') }
    try {
        $env:PATH = "$env:SystemRoot\System32;$env:SystemRoot"
        $env:CPU_EXPRESS_OFFLINE = '1'
        $process = Start-Process -FilePath $exe -ArgumentList @('--demo','--config',('"'+$config+'"'),'--smoke-seconds','8','--diagnostics',('"'+$diagnostics+'"')) -WorkingDirectory $portable -WindowStyle Hidden -PassThru
        if (-not $process.WaitForExit(30000)) { throw "Timeout do teste isolado $($process.Id)." }
        if ($process.ExitCode -ne 0) { throw 'Portátil falhou.' }
    } finally {
        $env:PATH=$oldPath
        $env:CPU_EXPRESS_OFFLINE=$null
        foreach ($key in $saved.Keys) { [Environment]::SetEnvironmentVariable($key,$saved[$key],'Process') }
    }
    $records = @(Get-Content $diagnostics -Encoding UTF8 | ForEach-Object { $_ | ConvertFrom-Json })
    if ($records[-1].event -ne 'shutdown' -or $records[-1].app_version -ne '0.4.0-rc.3') { throw 'Encerramento/versão não confirmado.' }
    Copy-Item -LiteralPath $diagnostics -Destination (Join-Path $evidence 'portable.jsonl')
    Write-Json (Join-Path $evidence 'portable-result.json') ([ordered]@{
        exit_code=$process.ExitCode; binary_sha256=(Get-Digest $exe); version=$info.version
        scope='Três ZIPs extraídos em caminho com espaços; hashes conferidos; perfil isolado; PATH somente Windows; offline'
        archive_payload_hashes_verified=$true; final_event=$records[-1].event; directory=$run; executable=$exe
    })
}
Write-Json (Join-Path $run 'integrity-result.json') ([ordered]@{
    exit_code=0; directory=$run; binary_sha256=(Get-Digest $exe); integrity_only=[bool]$IntegrityOnly
    archives=@(@('','-Fonte','-Evidencias') | ForEach-Object { $p=Join-Path $root "release\$name$_.zip"; @{name=(Split-Path $p -Leaf); sha256=(Get-Digest $p)} })
})
Write-Output $run
