# Removes only verified manifest-listed files, using the Recycle Bin.
# Example: .\uninstall-local.ps1 -WhatIf
[CmdletBinding(SupportsShouldProcess = $true)]
param()

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

if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) { throw 'No local installation manifest found; nothing will be removed.' }
Assert-NoReparse $manifestPath
$manifestHash = (Get-FileHash -LiteralPath $manifestPath -Algorithm SHA256).Hash
$manifest = Get-Content -LiteralPath $manifestPath -Raw -Encoding UTF8 | ConvertFrom-Json
if ($manifest.schema -ne 1 -or $manifest.application -cne 'CPU Express' -or
    -not [string]::Equals($manifest.installRoot, $installRoot, [StringComparison]::OrdinalIgnoreCase)) { throw 'Invalid installation manifest identity/root.' }
$files = @()
$seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
foreach ($entry in $manifest.files) {
    $target = Get-OwnedFilePath $entry.relative
    if ($entry.relative -eq $manifestName -or -not $seen.Add($target) -or $entry.sha256 -notmatch '^[a-fA-F0-9]{64}$') { throw 'Invalid or duplicate manifest file entry.' }
    if ((Test-Path -LiteralPath $target) -and -not (Test-Path -LiteralPath $target -PathType Leaf)) { throw "Manifest file is now a directory: $target" }
    $files += [pscustomobject]@{ path = $target; sha256 = $entry.sha256 }
}
$shortcuts = @($manifest.shortcuts | ForEach-Object { Get-ShortcutPath $_ })
if (-not $PSCmdlet.ShouldProcess($installRoot, 'Recycle unchanged manifest-listed application files; remove owned shortcuts/startup entry')) { return }
Add-Type -AssemblyName Microsoft.VisualBasic
function Move-FileToRecycleBin([string]$Path) {
    # No recursive directory removal and no permanent-delete fallback.
    [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile($Path,
        [Microsoft.VisualBasic.FileIO.UIOption]::OnlyErrorDialogs,
        [Microsoft.VisualBasic.FileIO.RecycleOption]::SendToRecycleBin,
        [Microsoft.VisualBasic.FileIO.UICancelOption]::ThrowException)
}
$preserved = $false
foreach ($file in $files) {
    Assert-InstallRoot
    Assert-AppStopped
    Assert-NoReparse $file.path
    if (-not (Test-Path -LiteralPath $file.path)) { continue }
    if ((Get-FileHash -LiteralPath $file.path -Algorithm SHA256).Hash -ne $file.sha256) {
        Write-Warning "Modified file preserved: $($file.path)"
        $preserved = $true
        continue
    }
    Move-FileToRecycleBin $file.path
}
foreach ($path in $shortcuts) {
    if ((Test-Path -LiteralPath $path) -and (Test-OwnedShortcut $path)) {
        $before = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
        if ((Test-OwnedShortcut $path) -and (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -eq $before) {
            Move-FileToRecycleBin $path
        }
    }
}
if ($manifest.autoStart -and (Test-Path -LiteralPath $runKey)) {
    $existing = Get-ItemProperty -LiteralPath $runKey
    $property = $existing.PSObject.Properties[$runName]
    if ($null -ne $property -and $property.Value -ceq $runCommand) {
        Remove-ItemProperty -LiteralPath $runKey -Name $runName
    }
}
Assert-InstallRoot
Assert-NoReparse $manifestPath
if (-not $preserved -and (Get-FileHash -LiteralPath $manifestPath -Algorithm SHA256).Hash -eq $manifestHash) {
    Move-FileToRecycleBin $manifestPath
}
Write-Output 'Uninstall completed. Modified/unlisted files and directories are preserved.'
Write-Output 'Saves under LOCALAPPDATA\CPU Express were not changed.'
