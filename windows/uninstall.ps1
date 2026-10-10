# Removes the Sinhala TSF text service. Run as Administrator.
$ErrorActionPreference = 'Continue'
$dest = Join-Path $env:ProgramFiles 'SinhalaInput'
& "$env:WINDIR\System32\regsvr32.exe" /u /s (Join-Path $dest 'sinhala_tsf_x64.dll')
& "$env:WINDIR\SysWOW64\regsvr32.exe" /u /s (Join-Path $dest 'sinhala_tsf_x86.dll')
Remove-Item -Recurse -Force $dest -ErrorAction SilentlyContinue
Write-Host "Removed. Sign out and back in to clear it from the language bar."
