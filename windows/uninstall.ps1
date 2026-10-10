# Removes the Sinhala TSF text service installed by install.ps1. Needs Administrator.
$ErrorActionPreference = 'Continue'
$dest = Join-Path $env:ProgramFiles 'SinhalaInput'
& "$env:WINDIR\System32\regsvr32.exe" /u /s (Join-Path $dest 'sinhala_tsf_x64.dll')
& "$env:WINDIR\SysWOW64\regsvr32.exe" /u /s (Join-Path $dest 'sinhala_tsf_x86.dll')
Remove-Item -Recurse -Force $dest -ErrorAction SilentlyContinue
if (Test-Path $dest) {
    Write-Warning "Some files are still in use. Sign out and back in, then delete $dest."
} else {
    Write-Host "Removed. Sign out and back in to clear it from the language list." -ForegroundColor Green
}
Write-Host "Your settings are kept in $env:APPDATA\SinhalaInput (delete that folder to reset)."
