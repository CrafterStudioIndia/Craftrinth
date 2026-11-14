; Craftrinth Windows 7 Installer Script
; Inno Setup 6.x Compatible

#define MyAppName "Craftrinth"
#define MyAppVersion "1.0.0-win7"
#define MyAppPublisher "CrafterStudio"
#define MyAppURL "https://github.com/CrafterStudioIndia/Craftrinth"
#define MyAppExeName "craftrinth_win7.exe"

[Setup]
AppId={{D8A1237A-93F1-4F29-873B-764E8C3B1234}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\Craftrinth
DisableProgramGroupPage=yes
; Windows 7 SP1 minimum version requirement (6.1.7601)
MinVersion=6.1.7601
OutputDir=dist
OutputBaseFilename=Craftrinth-Windows7-Setup
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "craftrinth_win7.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "README.md"; DestDir: "{app}"; Flags: isreadme
; PrimeAuthLib agents
Source: "..\packages\app-lib\resources\primeauthlib\*"; DestDir: "{userappdata}\.craftrinth\launcher\agents"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
