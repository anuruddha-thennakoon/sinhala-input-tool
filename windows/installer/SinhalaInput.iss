; Single-file setup.exe (Inno Setup 6). Same payload and registration as the MSI and install.ps1:
; the 64-bit and 32-bit text-service DLLs in %ProgramFiles%\SinhalaInput, each registered through
; its own DllRegisterServer. Build (CI does this):
;   iscc /DVersion=0.1.0 /DX64Dll=<path> /DX86Dll=<path> /DIcon=<path> /DOutDir=<dir> SinhalaInput.iss
[Setup]
AppId={{7B0E2C1A-5D6F-4E7B-9A31-2F4C8D1E6B50}
AppName=Sinhala Input
AppVersion={#Version}
AppPublisher=Itrowin
DefaultDirName={autopf}\SinhalaInput
DisableDirPage=yes
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#OutDir}
OutputBaseFilename=SinhalaInput-{#Version}-Setup
SetupIconFile={#Icon}
UninstallDisplayIcon={app}\sinhala.ico
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes

[Files]
Source: "{#X64Dll}"; DestDir: "{app}"; DestName: "sinhala_tsf_x64.dll"; Flags: ignoreversion regserver restartreplace uninsrestartdelete
Source: "{#X86Dll}"; DestDir: "{app}"; DestName: "sinhala_tsf_x86.dll"; Flags: ignoreversion regserver 32bit restartreplace uninsrestartdelete
Source: "{#Icon}"; DestDir: "{app}"; DestName: "sinhala.ico"; Flags: ignoreversion

[Messages]
FinishedLabel=Sinhala Input is installed.%n%nNext: Settings > Time & language > Language & region > add Sinhala, then Language options > Add a keyboard > "Sinhala Input". Switch with Win+Space. Restart Word (or any app that was already open).
