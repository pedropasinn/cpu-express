. "$PSScriptRoot\rc3-common.ps1"
Initialize-Toolchain
New-Item -ItemType Directory -Force $evidence | Out-Null
Push-Location $project
try {
    $sources = Get-SourceHashes
    $ErrorActionPreference = 'Continue' # PowerShell 5 trata stderr nativo como ErrorRecord.
    & cargo build --offline --locked --release -j 2 *> (Join-Path $evidence 'build.log')
    $ErrorActionPreference = 'Stop'
    if ($LASTEXITCODE -ne 0) { throw 'Build falhou; consulte evidence/rc3/build.log.' }
    Assert-SourceHashes ($sources | ConvertTo-Json -Depth 4 | ConvertFrom-Json)
    Write-Json (Join-Path $evidence 'build.json') ([ordered]@{
        exit_code=0; binary_sha256=(Get-Digest $binary); sources=$sources
        rustc=(& rustc -Vv | Out-String).Trim(); base_commit=(& git rev-parse HEAD)
        built_utc=[DateTime]::UtcNow.ToString('o'); jobs=2; offline=$true
    })
    & powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-current.ps1 -Jobs 2 -EvidenceDirectory evidence/rc3
    if ($LASTEXITCODE -ne 0) { throw 'Gates falharam.' }
    Assert-SourceHashes ($sources | ConvertTo-Json -Depth 4 | ConvertFrom-Json)
    Write-Json (Join-Path $evidence 'checked-sources.json') $sources
} finally { Pop-Location }
