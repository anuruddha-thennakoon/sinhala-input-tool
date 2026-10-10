# Installs the Sinhala TSF text service (64-bit and 32-bit) for all users. Run as Administrator.
#   cargo build --release -p sinhala-tsf --target x86_64-pc-windows-msvc
#   cargo build --release -p sinhala-tsf --target i686-pc-windows-msvc
#   powershell -ExecutionPolicy Bypass -File windows\install.ps1
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$dest = Join-Path $env:ProgramFiles 'SinhalaInput'
New-Item -ItemType Directory -Force -Path $dest | Out-Null

$dll64 = Join-Path $root 'target\x86_64-pc-windows-msvc\release\sinhala_tsf.dll'
$dll32 = Join-Path $root 'target\i686-pc-windows-msvc\release\sinhala_tsf.dll'
if (-not (Test-Path $dll64)) { throw "Build the 64-bit DLL first: $dll64" }

Copy-Item $dll64 (Join-Path $dest 'sinhala_tsf_x64.dll') -Force
& "$env:WINDIR\System32\regsvr32.exe" /s (Join-Path $dest 'sinhala_tsf_x64.dll')
if ($LASTEXITCODE -ne 0) { throw "64-bit registration failed ($LASTEXITCODE)" }

if (Test-Path $dll32) {
    Copy-Item $dll32 (Join-Path $dest 'sinhala_tsf_x86.dll') -Force
    & "$env:WINDIR\SysWOW64\regsvr32.exe" /s (Join-Path $dest 'sinhala_tsf_x86.dll')
    if ($LASTEXITCODE -ne 0) { throw "32-bit registration failed ($LASTEXITCODE)" }
} else {
    Write-Warning "No 32-bit DLL found; 32-bit apps will not see the input method."
}
Write-Host "Installed to $dest."
Write-Host "Now: Settings > Time & language > Language & region > add Sinhala, then enable the 'Sinhala Input' keyboard, and switch with Win+Space."
