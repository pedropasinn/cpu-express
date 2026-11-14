# Optional per-user installation; never touches LOCALAPPDATA\CPU Express saves.
# Example: .\install-local.ps1 -SourcePath C:\Packages\CPU-Express-0.4.0-rc.2 -StartMenu -Desktop
[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [Parameter(Mandatory = $true)][string]$SourcePath,
    [switch]$StartMenu,
    [switch]$Desktop,
    [switch]$AutoStart
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$manifestName = '.cpu-express-install.json'
$runKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$runName = 'CPU Express'
if (-not $env:LOCALAPPDATA) { throw 'LOCALAPPDATA is unavailable.' }
$installRoot = [IO.Path]::GetFullPath((Join-Path $env:LOCALAPPDATA 'Programs\CPU Express')).TrimEnd('\')
$exePath = Join-Path $installRoot 'cpu-express.exe'
$runCommand = '"' + $exePath + '"'
$manifestPath = Join-Path $installRoot $manifestName

function Assert-NoReparse([string]$Path) {
    $cursor = [IO.Path]::GetFullPath($Path)
    while ($cursor) {
        if (Test-Path -LiteralPath $cursor) {
            $item = Get-Item -LiteralPath $cursor -Force
            if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) {
                throw "Reparse points are not supported: $cursor"
            }
        }
        $parent = [IO.Path]::GetDirectoryName($cursor)
        if ($parent -eq $cursor) { break }
        $cursor = $parent
    }
}
function Assert-InstallRoot {
    Assert-NoReparse $installRoot
    if (Test-Path -LiteralPath $installRoot) {
        $resolved = (Resolve-Path -LiteralPath $installRoot).ProviderPath.TrimEnd('\')
        if (-not [string]::Equals($resolved, $installRoot, [StringComparison]::OrdinalIgnoreCase)) {
            throw 'Resolved installation target differs from the exact allowed root.'
        }
    }
}
function Assert-AppStopped {
    # Fail closed if process enumeration cannot complete. Never stop a process.
    $processes = @(Get-CimInstance Win32_Process -Filter "Name = 'cpu-express.exe'")
    foreach ($process in $processes) {
        if (-not $process.ExecutablePath) {
            throw 'Cannot identify a running CPU Express process. Close CPU Express and retry.'
        }
        $path = [IO.Path]::GetFullPath($process.ExecutablePath)
        if ($path.StartsWith($installRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
            throw "Close CPU Express running under $installRoot and retry."
        }
    }
}
function Get-OwnedFilePath([string]$Relative) {
    if ([string]::IsNullOrWhiteSpace($Relative) -or [IO.Path]::IsPathRooted($Relative) -or
        $Relative.Contains(':') -or ($Relative -split '[\\/]' | Where-Object { $_ -in @('', '.', '..') -or $_.EndsWith('.') -or $_.EndsWith(' ') })) {
        throw "Unsafe manifest relative path: $Relative"
    }
    $full = [IO.Path]::GetFullPath((Join-Path $installRoot $Relative))
    if (-not $full.StartsWith($installRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Manifest path escapes the installation root.'
    }
    Assert-NoReparse $full
    return $full
}
function Get-ShortcutPath([string]$Kind) {
    switch ($Kind) {
        'StartMenu' { $folder = [Environment]::GetFolderPath('Programs') }
        'Desktop' { $folder = [Environment]::GetFolderPath('DesktopDirectory') }
        default { throw 'Invalid shortcut kind.' }
    }
    if (-not $folder) { throw "Unavailable shortcut folder: $Kind" }
    $path = Join-Path $folder 'CPU Express.lnk'
    Assert-NoReparse $path
    return $path
}
function Test-OwnedShortcut([string]$Path) {
    Assert-NoReparse $Path
    $shell = New-Object -ComObject WScript.Shell
    try {
        $shortcut = $shell.CreateShortcut($Path)
        try {
            return ([string]::Equals($shortcut.TargetPath, $exePath, [StringComparison]::OrdinalIgnoreCase) -and
                [string]::IsNullOrEmpty($shortcut.Arguments))
        } finally { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($shortcut) }
    } finally { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($shell) }
}
Assert-InstallRoot
Assert-AppStopped

$source = (Resolve-Path -LiteralPath $SourcePath).ProviderPath.TrimEnd('\')
Assert-NoReparse $source
if (-not (Test-Path -LiteralPath $source -PathType Container)) { throw 'SourcePath must be an extracted portable release directory.' }
if ($source -eq $installRoot -or $installRoot.StartsWith($source + '\', [StringComparison]::OrdinalIgnoreCase) -or
    $source.StartsWith($installRoot + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Source and installation directories must be separate.' }
if (-not (Test-Path -LiteralPath (Join-Path $source 'cpu-express.exe') -PathType Leaf)) { throw 'Portable release must contain cpu-express.exe at its root.' }
# Empty directories left by uninstall are reusable, but no existing file is
# overwritten, including files unknown to a previous installation manifest.
$existingDirectories = [Collections.Generic.Stack[string]]::new()
if (Test-Path -LiteralPath $installRoot) { $existingDirectories.Push($installRoot) }
while ($existingDirectories.Count) {
    foreach ($item in Get-ChildItem -LiteralPath $existingDirectories.Pop() -Force) {
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Reparse point in installation: $($item.FullName)" }
        if (-not $item.PSIsContainer) {
            throw 'Installation contains files. Uninstall first and preserve any modified/unlisted files elsewhere; saves are preserved.'
        }
        $existingDirectories.Push($item.FullName)
    }
}
# Enumerate one level at a time; reject junctions before descending into them.
$pending = [Collections.Generic.Stack[string]]::new()
$pending.Push($source)
$files = @()
while ($pending.Count) {
    $directory = $pending.Pop()
    foreach ($item in Get-ChildItem -LiteralPath $directory -Force) {
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Reparse point in portable source: $($item.FullName)" }
        if ($item.PSIsContainer) { $pending.Push($item.FullName); continue }
        $relative = $item.FullName.Substring($source.Length + 1)
        if ($relative -eq $manifestName) { throw 'Source contains a local installation manifest; use an original portable release.' }
        $target = Get-OwnedFilePath $relative
        $files += [pscustomobject]@{ relative = $relative; sha256 = (Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash; source = $item.FullName; target = $target }
    }
}
$shortcuts = @()
if ($StartMenu) { $shortcuts += 'StartMenu' }
if ($Desktop) { $shortcuts += 'Desktop' }
foreach ($kind in $shortcuts) {
    $path = Get-ShortcutPath $kind
    if (Test-Path -LiteralPath $path) { throw "Shortcut already exists; refusing to replace it: $path" }
}
if ($AutoStart -and (Test-Path -LiteralPath $runKey)) {
    $existing = Get-ItemProperty -LiteralPath $runKey
    $property = $existing.PSObject.Properties[$runName]
    if ($null -ne $property -and $property.Value -cne $runCommand) { throw 'An unrelated CPU Express Run entry already exists.' }
}
if (-not $PSCmdlet.ShouldProcess($installRoot, 'Install portable files and explicitly requested shortcuts/startup entry')) { return }
New-Item -ItemType Directory -Path $installRoot -Force | Out-Null
Assert-InstallRoot
$manifest = [ordered]@{
    schema = 1
    application = 'CPU Express'
    installRoot = $installRoot
    files = @($files | ForEach-Object { [ordered]@{ relative = $_.relative; sha256 = $_.sha256 } })
    shortcuts = $shortcuts
    autoStart = [bool]$AutoStart
}
# Record intended paths before copying so interrupted installs can be inspected
# and uninstalled safely. Removal still requires each file's hash to match.
$manifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $manifestPath -Encoding UTF8
foreach ($file in $files) {
    Assert-InstallRoot
    $target = Get-OwnedFilePath $file.relative
    if (Test-Path -LiteralPath $target) { throw "Destination appeared during installation: $target" }
    Assert-NoReparse $file.source
    New-Item -ItemType Directory -Path ([IO.Path]::GetDirectoryName($target)) -Force | Out-Null
    Copy-Item -LiteralPath $file.source -Destination $target
    if ((Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash -ne $file.sha256) { throw "Copy verification failed: $target" }
}
foreach ($kind in $shortcuts) {
    $path = Get-ShortcutPath $kind
    if (Test-Path -LiteralPath $path) { throw "Shortcut appeared during installation: $path" }
    $shell = New-Object -ComObject WScript.Shell
    try {
        $shortcut = $shell.CreateShortcut($path)
        try {
            $shortcut.TargetPath = $exePath
            $shortcut.WorkingDirectory = $installRoot
            $shortcut.Description = 'CPU Express'
            $shortcut.Save()
        } finally { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($shortcut) }
    } finally { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($shell) }
}
if ($AutoStart) {
    if (-not (Test-Path -LiteralPath $runKey)) {
        New-Item -Path $runKey | Out-Null
    }
    $existing = Get-ItemProperty -LiteralPath $runKey
    $property = $existing.PSObject.Properties[$runName]
    if ($null -ne $property -and $property.Value -cne $runCommand) { throw 'Run entry changed during installation; it was not replaced.' }
    New-ItemProperty -LiteralPath $runKey -Name $runName -Value $runCommand -PropertyType String -Force | Out-Null
}
Write-Output "Installed: $exePath"
Write-Output 'Saves under LOCALAPPDATA\CPU Express were not changed.'
