#Requires -Version 5.1
<#
.SYNOPSIS
Validate, compile and package CPU Express on Windows with locked dependencies.
.DESCRIPTION
Requires an installed stable Rust toolchain, rustfmt, clippy, the chosen Windows
target and its linker/SDK. Nothing is installed automatically. Cargo may fetch
the versions already locked in Cargo.lock; the resulting app has no network
service or browser dependency. No CPU stress or load-generation test is run.
.EXAMPLE
.\scripts\build-windows.ps1
.EXAMPLE
.\scripts\build-windows.ps1 -Target x86_64-pc-windows-gnu -OutputDir C:\Builds
#>
[CmdletBinding()]
param(
    [ValidateSet('x86_64-pc-windows-msvc', 'x86_64-pc-windows-gnu')]
    [string]$Target = 'x86_64-pc-windows-msvc',
    [string]$OutputDir
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'Run this build script on Windows to execute the native test binaries.'
}
if ($null -eq (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Cargo was not found. Install the Rust toolchain and Windows build prerequisites described in README.md.'
}
$projectRoot = Split-Path -Parent $PSScriptRoot
if (-not (Test-Path -LiteralPath (Join-Path $projectRoot 'Cargo.lock') -PathType Leaf)) {
    throw 'Cargo.lock is missing. Use the complete source package.'
}
if ([string]::IsNullOrWhiteSpace($OutputDir)) { $OutputDir = Join-Path $projectRoot 'dist' }
$outputRoot = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputDir)

function Test-LicenseInventory([object]$Metadata, [string]$TargetTriple, [string]$RustVersion) {
    $licenseRoot = Join-Path $projectRoot 'licenses'
    $manifestPath = Join-Path $licenseRoot 'manifest.json'
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        throw 'licenses/manifest.json is missing. Use the consolidated source with third-party notices.'
    }
    $manifest = Get-Content -LiteralPath $manifestPath -Raw -Encoding UTF8 | ConvertFrom-Json
    $inventory = @{}
    foreach ($entry in $manifest.packages) { $inventory[('{0}@{1}' -f $entry.name, $entry.version)] = $entry }
    $resolved = @{}
    foreach ($node in $Metadata.resolve.nodes) { $resolved[$node.id] = $true }
    $covered = @()
    foreach ($package in $Metadata.packages) {
        if (-not $resolved.ContainsKey($package.id) -or $null -eq $package.source -or -not $package.source.StartsWith('registry+')) { continue }
        $key = '{0}@{1}' -f $package.name, $package.version
        if (-not $inventory.ContainsKey($key)) { throw "Missing license inventory entry: $key. Regenerate notices for $TargetTriple." }
        $entry = $inventory[$key]
        if ($entry.registry -ne $package.source -or $entry.license -ne $package.license) {
            throw "License metadata does not match the locked package: $key"
        }
        if (@($entry.files).Count -eq 0) { throw "No license text for $key" }
        $covered += $key
    }
    # The copied toolchain notices retain their original source labels. Require
    # the same Rust release instead of silently attributing old notices to a new std.
    $rustNotices = @($manifest.toolchain_files | Where-Object { $_.path.StartsWith('toolchain/rust/') })
    if ($rustNotices.Count -eq 0) { throw 'Rust standard-library notices are missing.' }
    foreach ($notice in $rustNotices) {
        if (-not $notice.source.StartsWith("Rust $RustVersion distribution ")) {
            throw "Supplied Rust notices do not describe toolchain $RustVersion. Regenerate notices before packaging."
        }
    }
    # Verify every supplied notice, including optional GNU toolchain notices.
    $allNotices = @($manifest.packages | ForEach-Object { $_.files }) + @($manifest.toolchain_files)
    $canonicalRoot = [IO.Path]::GetFullPath($licenseRoot).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    foreach ($notice in $allNotices) {
        $noticePath = [IO.Path]::GetFullPath((Join-Path $licenseRoot $notice.path))
        if (-not $noticePath.StartsWith($canonicalRoot, [StringComparison]::OrdinalIgnoreCase)) { throw 'A license path escapes licenses/.' }
        if (-not (Test-Path -LiteralPath $noticePath -PathType Leaf)) { throw "Missing notice: $($notice.path)" }
        if ((Get-Item -LiteralPath $noticePath).Length -ne $notice.bytes -or
            (Get-FileHash -LiteralPath $noticePath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $notice.sha256) {
            throw "Changed or corrupt notice: $($notice.path)"
        }
    }
    return [ordered]@{
        target = $TargetTriple
        rust_version = $RustVersion
        source_inventory_target = $manifest.target
        registry_packages_checked = $covered.Count
        registry_packages = $covered
        supplied_notice_files_checked = $allNotices.Count
        dependency_notice_coverage = 'Every registry package in the current resolved target graph has matching metadata and intact notice files.'
        toolchain_notice_scope = 'Rust notices match the compiler release. GNU toolchain notices retain their original provenance; MSVC/SDK licensing is not inferred from the GNU inventory.'
        cargo_lock_sha256 = (Get-FileHash -LiteralPath Cargo.lock -Algorithm SHA256).Hash.ToLowerInvariant()
        scope = 'Presence, version and byte-integrity check; not a binary-symbol audit.'
    }
}

function Invoke-CargoChecked([string[]]$CargoArguments) {
    Write-Host ('cargo ' + ($CargoArguments -join ' '))
    & cargo @CargoArguments
    if ($LASTEXITCODE -ne 0) { throw ('Cargo failed: ' + ($CargoArguments -join ' ')) }
}

Push-Location -LiteralPath $projectRoot
try {
    $lockBefore = (Get-FileHash -LiteralPath Cargo.lock -Algorithm SHA256).Hash
    Invoke-CargoChecked -CargoArguments @('fmt', '--all', '--', '--check')
    Invoke-CargoChecked -CargoArguments @('clippy', '--locked', '--all-targets', '--target', $Target, '--', '-D', 'warnings')
    Invoke-CargoChecked -CargoArguments @('test', '--locked', '--target', $Target)
    Invoke-CargoChecked -CargoArguments @('build', '--locked', '--release', '--target', $Target)

    $metadataText = & cargo metadata --locked --format-version 1 --filter-platform $Target
    if ($LASTEXITCODE -ne 0) { throw 'Could not read Cargo build metadata.' }
    $metadata = $metadataText | ConvertFrom-Json
    if ((Get-FileHash -LiteralPath Cargo.lock -Algorithm SHA256).Hash -ne $lockBefore) { throw 'Cargo.lock changed during validation.' }
    $rustVersion = & rustc --version --verbose
    if ($LASTEXITCODE -ne 0) { throw 'Could not read the compiler version.' }
    $releaseLine = @($rustVersion | Where-Object { $_ -match '^release: ' })
    if ($releaseLine.Count -ne 1) { throw 'Could not identify the Rust release.' }
    $rustRelease = $releaseLine[0].Substring('release: '.Length).Trim()
    $licenseAudit = Test-LicenseInventory -Metadata $metadata -TargetTriple $Target -RustVersion $rustRelease
    $package = $metadata.packages | Where-Object { $_.name -eq 'cpu-express' } | Select-Object -First 1
    if ($null -eq $package) { throw 'The cpu-express package was not present in Cargo metadata.' }
    $binary = Join-Path $metadata.target_directory (Join-Path $Target 'release\cpu-express.exe')
    if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) {
        throw "Cargo completed but the executable was not found: $binary"
    }
    $releaseName = 'cpu-express-{0}-{1}-{2}-{3}' -f $package.version, $Target, (Get-Date -Format 'yyyyMMdd-HHmmss'), ([guid]::NewGuid().ToString('N').Substring(0, 8))
    $releaseDirectory = Join-Path $outputRoot $releaseName
    [void][IO.Directory]::CreateDirectory($releaseDirectory)
    $packagedExe = Join-Path $releaseDirectory 'cpu-express.exe'
    Copy-Item -LiteralPath $binary -Destination $packagedExe
    foreach ($name in @('LICENSE', 'README.md', 'LEIA-ME.txt', 'LICENSE-INVENTORY.md', 'LICENSE-INVENTORY.json', 'THIRD_PARTY_NOTICES.md', 'THIRD_PARTY_NOTICES.html', 'THIRD_PARTY_LICENSES.md', 'NOTICE')) {
        $source = Join-Path $projectRoot $name
        if (Test-Path -LiteralPath $source -PathType Leaf) {
            Copy-Item -LiteralPath $source -Destination (Join-Path $releaseDirectory $name)
        }
    }
    foreach ($directoryName in @('licenses', 'assets')) {
        $sourceDirectory = Join-Path $projectRoot $directoryName
        if (Test-Path -LiteralPath $sourceDirectory -PathType Container) {
            Copy-Item -LiteralPath $sourceDirectory -Destination (Join-Path $releaseDirectory $directoryName) -Recurse
        }
    }
    $measurementScript = Join-Path $PSScriptRoot 'measure-windows.ps1'
    if (Test-Path -LiteralPath $measurementScript -PathType Leaf) {
        [void][IO.Directory]::CreateDirectory((Join-Path $releaseDirectory 'scripts'))
        Copy-Item -LiteralPath $measurementScript -Destination (Join-Path $releaseDirectory 'scripts\measure-windows.ps1')
    }
    [void][IO.Directory]::CreateDirectory((Join-Path $releaseDirectory 'docs'))
    foreach ($document in @('windows_checklist.md', 'performance_windows.md', 'ux_guia.md')) {
        $source = Join-Path $projectRoot ('docs/' + $document)
        if (Test-Path -LiteralPath $source -PathType Leaf) { Copy-Item -LiteralPath $source -Destination (Join-Path $releaseDirectory ('docs/' + $document)) }
    }
    $buildInfo = [ordered]@{
        application_version = $package.version; target = $Target; profile = 'release'
        built_utc = [DateTime]::UtcNow.ToString('o')
        rustc = ($rustVersion -join [Environment]::NewLine)
        operating_system = [Environment]::OSVersion.VersionString
        executable_bytes = (Get-Item -LiteralPath $packagedExe).Length
        executable_sha256 = (Get-FileHash -LiteralPath $packagedExe -Algorithm SHA256).Hash.ToLowerInvariant()
        validation = @('cargo fmt --all -- --check', 'cargo clippy --locked --all-targets --target TARGET -- -D warnings', 'cargo test --locked --target TARGET', 'cargo build --locked --release --target TARGET')
        cargo_lock_sha256 = $lockBefore.ToLowerInvariant()
        signed = $false
        integrated_fronts = @('Windows', 'Art', 'World', 'Metrics', 'UX')
        gui_validation = 'Not executed by this script. Open cpu-express.exe and follow docs/windows_checklist.md on Windows 10/11.'
        license_validation = $licenseAudit
    }
    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    [IO.File]::WriteAllText((Join-Path $releaseDirectory 'BUILD-INFO.json'), ($buildInfo | ConvertTo-Json -Depth 5), $utf8NoBom)
    [IO.File]::WriteAllText((Join-Path $releaseDirectory 'LICENSE-AUDIT.json'), ($licenseAudit | ConvertTo-Json -Depth 6), $utf8NoBom)
    @((Get-FileHash -LiteralPath $packagedExe -Algorithm SHA256).Hash.ToLowerInvariant() + '  cpu-express.exe') |
        Set-Content -LiteralPath (Join-Path $releaseDirectory 'SHA256SUMS.txt') -Encoding ascii
    @(
        'CPU Express - cinco frentes integradas'
        ''
        'Execute cpu-express.exe. Para uma demonstracao sem coleta real: cpu-express.exe --demo'
        'Ocultar na bandeja permite recuperar o trem pelo icone. Sair encerra o processo.'
        'Este executavel foi compilado e submetido aos testes automatizados desta fonte.'
        'A aceitacao visual em Windows 10/11 ainda exige o checklist manual em docs/windows_checklist.md.'
        'BUILD-INFO.json descreve esta compilacao. Nenhum log de um executavel anterior comprova este pacote.'
    ) | Set-Content -LiteralPath (Join-Path $releaseDirectory 'LEIA-ME-EXECUTAVEL.txt') -Encoding utf8
    $archive = Join-Path $outputRoot ($releaseName + '.zip')
    $items = @(Get-ChildItem -LiteralPath $releaseDirectory | ForEach-Object { $_.FullName })
    Compress-Archive -LiteralPath $items -DestinationPath $archive -CompressionLevel Optimal
    Write-Host "Executable: $packagedExe"
    Write-Host "Portable archive: $archive"
    Write-Host ('Archive SHA-256: ' + (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant())
} finally {
    Pop-Location
}
