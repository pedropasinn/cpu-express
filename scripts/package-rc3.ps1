param([switch]$Preliminary)
. "$PSScriptRoot\rc3-common.ps1"
Add-Type -AssemblyName System.IO.Compression.FileSystem
Add-Type -AssemblyName System.IO.Compression
function Read-Json($Name) { Get-Content -LiteralPath (Join-Path $evidence $Name) -Raw -Encoding UTF8 | ConvertFrom-Json }
function New-Zip($Path, $Files, $Base, $Prefix) {
    # Somente arquivos de release podem ser substituídos. Nenhuma limpeza da fonte.
    $releaseRoot = [IO.Path]::GetFullPath((Join-Path $root 'release')) + '\'
    if (-not [IO.Path]::GetFullPath($Path).StartsWith($releaseRoot, [StringComparison]::OrdinalIgnoreCase)) { throw 'ZIP fora de release.' }
    $stream = [IO.File]::Open($Path, [IO.FileMode]::Create)
    $zip = New-Object IO.Compression.ZipArchive($stream, [IO.Compression.ZipArchiveMode]::Create)
    try {
        foreach ($file in $Files) {
            $relative = $file.Substring($Base.Length+1).Replace('\','/')
            [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $file, ($Prefix+$relative), [IO.Compression.CompressionLevel]::Optimal) | Out-Null
        }
    } finally { $zip.Dispose(); $stream.Dispose() }
}
$build = Read-Json 'build.json'
Assert-SourceHashes $build.sources
Assert-SourceHashes (Read-Json 'checked-sources.json')
$hash = Get-Digest $binary
if ($build.binary_sha256 -ne $hash) { throw 'Binário diferente do build validado.' }
$checks = Read-Json 'checks.json'
if ($checks.Count -ne 3 -or @($checks | Where-Object { $_.exit_code -ne 0 }).Count -gt 0 -or (@($checks.name | Sort-Object) -join ',') -ne 'clippy,format,tests') { throw 'Gates incompletos.' }
$native = [ordered]@{}
foreach ($name in @('native-smoke-result.json','portable-result.json','profile-migration-result.json')) {
    if (Test-Path (Join-Path $evidence $name)) {
        $result = Read-Json $name
        if ($result.binary_sha256 -ne $hash -or $result.exit_code -ne 0) { throw "Evidência inválida: $name" }
        $native[$name] = $result
    } elseif (-not $Preliminary) { throw "Evidência ausente: $name" }
}
$version = [regex]::Match((Get-Content (Join-Path $project 'Cargo.toml') -Raw), '(?m)^version = "([^"]+)"').Groups[1].Value
if ($version -ne '0.4.0-rc.3') { throw 'Este fluxo é específico da rc.3.' }
$release = Join-Path $root "release\CPU-Express-$version"
New-Item -ItemType Directory -Force $release | Out-Null
Copy-Item -LiteralPath $binary -Destination (Join-Path $release 'cpu-express.exe')
foreach ($name in @('README.md','LICENSE','THIRD_PARTY_NOTICES.md')) { Copy-Item -LiteralPath (Join-Path $project $name) -Destination $release }
foreach ($name in @('licenses','data','docs')) {
    $destination = Join-Path $release $name
    New-Item -ItemType Directory -Force $destination | Out-Null
    Get-ChildItem -LiteralPath (Join-Path $project $name) | Copy-Item -Destination $destination -Recurse -Force
}
foreach ($name in @('install-local.ps1','uninstall-local.ps1')) { Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination $release }
$imports = & (Join-Path $root '.tools\mingw64\bin\objdump.exe') -p $binary
if ($LASTEXITCODE -ne 0) { throw 'Falha ao inspecionar DLLs.' }
$dlls = @([regex]::Matches(($imports -join "`n"), 'DLL Name:\s*(\S+)') | ForEach-Object { $_.Groups[1].Value })
foreach ($dll in $dlls) {
    if ($dll -match '^lib(gcc|stdc|winpthread)') { Copy-Item -LiteralPath (Join-Path $root ".tools\mingw64\bin\$dll") -Destination $release }
}
$inventory = Get-Content (Join-Path $project 'licenses\manifest.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$notices = @($inventory.packages | ForEach-Object { $_.files }) + @($inventory.toolchain_files)
foreach ($notice in $notices) {
    if ((Get-Digest (Join-Path $project ('licenses/'+$notice.path))) -ne $notice.sha256) { throw "Licença divergente: $($notice.path)" }
}
$counts = [regex]::Matches((Get-Content (Join-Path $evidence 'tests.log') -Raw), 'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored')
if ($counts.Count -eq 0) { throw 'Sem resultados da suíte.' }
$totals = [ordered]@{passed=0;failed=0;ignored=0}
foreach ($count in $counts) { $totals.passed += [int]$count.Groups[1].Value; $totals.failed += [int]$count.Groups[2].Value; $totals.ignored += [int]$count.Groups[3].Value }
$info = [ordered]@{
    version=$version; status=$(if ($Preliminary) {'candidato local, validação de pacote pendente'} else {'beta local validado no escopo descrito'})
    target='x86_64-pc-windows-gnu'; profile='release'; signed=$false
    built_utc=$build.built_utc; rustc=$build.rustc; base_commit=$build.base_commit
    working_tree='alterações não commitadas; manifesto identifica a fonte exata'; executable_sha256=$hash
    executable_bytes=(Get-Item $binary).Length; checks=$checks; rust_tests=$totals
    production_and_test_sources=$build.sources; native=$native; imports=$dlls
    license_files_verified=$notices.Count; data_license='ODbL-1.0 / © OpenStreetMap contributors; procedência e demais fontes em data/sources/curitiba-paranagua/README.md'
    network='Compilação e ensaios offline; nenhum envio externo nesta rodada'
    not_validated=@('cliques/arrastes/compositor deste executável','Windows 10','outro computador/conta nova','DPI misto/segundo monitor','autohide/reinício do Explorer','SmartScreen real','clima online','soak/desempenho','workflow remoto','assinatura digital')
    visual_acceptance='Aceite anterior do Pedro no Windows 11 dele; não é nova evidência visual do binário rc.3.'
}
Write-Json (Join-Path $project 'BUILD-INFO.json') $info
Copy-Item -LiteralPath (Join-Path $project 'BUILD-INFO.json') -Destination $release
[IO.File]::WriteAllText((Join-Path $release 'LEIA-ME.txt'), "CPU Express $version`r`nExtraia todo o ZIP e abra cpu-express.exe.`r`nClique no trem para abrir a central; botão direito pausa.`r`nAcesso pela faixa e bandeja, sem botão na barra de tarefas.`r`nExecutável sem assinatura: veja docs/windows_primeira_abertura.md.`r`nRoteiro para amigos: docs/ROTEIRO_BETA.md.`r`nOs dados OSM em data/ têm licença ODbL-1.0 e crédito aos contribuidores do OpenStreetMap.`r`n", $utf8)
if (Test-Path (Join-Path $PSScriptRoot 'Iniciar_DIAGNOSTICO.cmd')) { Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'Iniciar_DIAGNOSTICO.cmd') -Destination $release }
$payload = @(Get-ChildItem $release -Recurse -File | Where-Object { $_.Name -ne 'SHA256SUMS.txt' } | Sort-Object FullName)
$lines = @($payload | ForEach-Object { (Get-Digest $_.FullName)+'  '+$_.FullName.Substring($release.Length+1).Replace('\','/') })
[IO.File]::WriteAllLines((Join-Path $release 'SHA256SUMS.txt'), $lines, $utf8)
New-Zip ($release+'.zip') @(Get-ChildItem $release -Recurse -File | ForEach-Object FullName) $release ((Split-Path $release -Leaf)+'/')
Push-Location $project
try { $sourceNames = @(& git ls-files --cached --others --exclude-standard | Where-Object { $_ -ne 'SOURCE-MANIFEST.json' }) } finally { Pop-Location }
$manifest = [ordered]@{}
foreach ($name in ($sourceNames | Sort-Object -Unique)) { $manifest[$name] = Get-Digest (Join-Path $project $name) }
Write-Json (Join-Path $project 'SOURCE-MANIFEST.json') $manifest
$sourceFiles = @($manifest.Keys | ForEach-Object { Join-Path $project $_ }) + @(Join-Path $project 'SOURCE-MANIFEST.json')
New-Zip ($release+'-Fonte.zip') $sourceFiles $project 'cpu-express/'
$evidenceManifest = [ordered]@{}
Get-ChildItem $evidence -Recurse -File | Where-Object { $_.Name -ne 'EVIDENCE-MANIFEST.json' } | Sort-Object FullName | ForEach-Object {
    $evidenceManifest[$_.FullName.Substring($evidence.Length+1).Replace('\','/')] = Get-Digest $_.FullName
}
Write-Json (Join-Path $evidence 'EVIDENCE-MANIFEST.json') $evidenceManifest
New-Zip ($release+'-Evidencias.zip') @(Get-ChildItem $evidence -Recurse -File | ForEach-Object FullName) $project ''
$archiveHashes = @('','-Fonte','-Evidencias') | ForEach-Object { $path=$release+$_+'.zip'; (Get-Digest $path)+'  '+(Split-Path $path -Leaf) }
[IO.File]::WriteAllLines(($release+'-SHA256SUMS.txt'), [string[]]$archiveHashes, $utf8)
Write-Output ($info | ConvertTo-Json -Depth 2)
