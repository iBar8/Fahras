; SPDX-License-Identifier: MIT
; Fahras installer. Build: iscc /DAppVersion=1.2.3 /DSourceDir=..\..\dist\Fahras /DOutputDir=..\..\dist fahras.iss

#ifndef AppVersion
  #error Pass /DAppVersion=<version>
#endif
#ifndef SourceDir
  #define SourceDir "..\..\dist\Fahras"
#endif
#ifndef OutputDir
  #define OutputDir "..\..\dist"
#endif

[Setup]
AppId={{03512DC5-FE1A-451D-A96E-15B5EB7C4D48}
AppName=Fahras
AppVersion={#AppVersion}
AppVerName=Fahras {#AppVersion}
AppPublisher=iBar8
AppPublisherURL=https://github.com/iBar8/Fahras
AppSupportURL=https://github.com/iBar8/Fahras/issues
AppUpdatesURL=https://github.com/iBar8/Fahras/releases
DefaultDirName={autopf}\Fahras
DefaultGroupName=Fahras
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputDir={#OutputDir}
OutputBaseFilename=Fahras-Setup-{#AppVersion}-x64
SetupIconFile=..\..\branding\fahras.ico
UninstallDisplayIcon={app}\bin\Fahras.exe
UninstallDisplayName=Fahras
LicenseFile=..\..\LICENSE
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
ShowLanguageDialog=auto
LanguageDetectionMethod=uilanguage

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "ar"; MessagesFile: "compiler:Languages\Arabic.isl"

[CustomMessages]
en.LaunchFahras=Open Fahras
ar.LaunchFahras=افتح فهرس
en.FahrasComment=Browse and manage your files
ar.FahrasComment=تصفح ملفاتك وأدرها

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[InstallDelete]
; Replace the runtime as a whole so files dropped by a newer GTK do not linger.
Type: filesandordirs; Name: "{app}\bin"
Type: filesandordirs; Name: "{app}\lib"
Type: filesandordirs; Name: "{app}\share"

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\Fahras"; Filename: "{app}\bin\Fahras.exe"; WorkingDir: "{userdocs}"; Comment: "{cm:FahrasComment}"
Name: "{autodesktop}\Fahras"; Filename: "{app}\bin\Fahras.exe"; WorkingDir: "{userdocs}"; Comment: "{cm:FahrasComment}"; Tasks: desktopicon

[Run]
Filename: "{app}\bin\Fahras.exe"; Description: "{cm:LaunchFahras}"; Flags: nowait postinstall skipifsilent
