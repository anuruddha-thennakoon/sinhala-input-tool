# Installs the Sinhala TSF text service (64-bit and 32-bit) for all users. Needs Administrator.
# Works from the portable zip (DLLs next to this script) or from a source checkout after:
#   cargo build --release -p sinhala-tsf --target x86_64-pc-windows-msvc
#   cargo build --release -p sinhala-tsf --target i686-pc-windows-msvc
$ErrorActionPreference = 'Stop'

$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "Run this as Administrator (right-click > Run as administrator), or use Install.cmd."
}

if (Test-Path (Join-Path $PSScriptRoot 'sinhala_tsf_x64.dll')) {      # portable zip layout
    $dll64 = Join-Path $PSScriptRoot 'sinhala_tsf_x64.dll'
    $dll32 = Join-Path $PSScriptRoot 'sinhala_tsf_x86.dll'
    $icon  = Join-Path $PSScriptRoot 'sinhala.ico'
} else {                                                              # source checkout layout
    $root  = Split-Path -Parent $PSScriptRoot
    $dll64 = Join-Path $root 'target\x86_64-pc-windows-msvc\release\sinhala_tsf.dll'
    $dll32 = Join-Path $root 'target\i686-pc-windows-msvc\release\sinhala_tsf.dll'
    $icon  = Join-Path $PSScriptRoot 'installer\sinhala.ico'
}
if (-not (Test-Path $dll64)) { throw "64-bit DLL not found: $dll64 (build it first)" }

$dest = Join-Path $env:ProgramFiles 'SinhalaInput'
New-Item -ItemType Directory -Force -Path $dest | Out-Null

# Unregister any previous version first, so its DLLs are not held by the registry entries.
foreach ($pair in @(@("$env:WINDIR\System32\regsvr32.exe", 'sinhala_tsf_x64.dll'),
                    @("$env:WINDIR\SysWOW64\regsvr32.exe", 'sinhala_tsf_x86.dll'))) {
    $old = Join-Path $dest $pair[1]
    if (Test-Path $old) { & $pair[0] /u /s $old }
}

try {
    if (Test-Path $icon) { Copy-Item $icon (Join-Path $dest 'sinhala.ico') -Force }
    Copy-Item $dll64 (Join-Path $dest 'sinhala_tsf_x64.dll') -Force
    if (Test-Path $dll32) { Copy-Item $dll32 (Join-Path $dest 'sinhala_tsf_x86.dll') -Force }
} catch {
    throw "Could not copy files (an app may still be using the old version). Close Word and other apps, or sign out and back in, then run this again. $($_.Exception.Message)"
}

& "$env:WINDIR\System32\regsvr32.exe" /s (Join-Path $dest 'sinhala_tsf_x64.dll')
if ($LASTEXITCODE -ne 0) { throw "64-bit registration failed (regsvr32 exit code $LASTEXITCODE)" }

if (Test-Path $dll32) {
    & "$env:WINDIR\SysWOW64\regsvr32.exe" /s (Join-Path $dest 'sinhala_tsf_x86.dll')
    if ($LASTEXITCODE -ne 0) { throw "32-bit registration failed (regsvr32 exit code $LASTEXITCODE)" }
} else {
    Write-Warning "No 32-bit DLL found; 32-bit apps will not see the input method."
}

Write-Host ""
Write-Host "Installed to $dest." -ForegroundColor Green
Write-Host "Next: Settings > Time & language > Language & region > add Sinhala, then"
Write-Host "      Language options > Add a keyboard > 'Sinhala Input'. Switch with Win+Space."
Write-Host "      Restart Word (or any app that was already open)."
