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
; Per user only: no administrator prompt to install or update, and the
; Open in Fahras and default-folder settings stay in this user's registry.
PrivilegesRequired=lowest
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
en.OpenInFahras=Open in Fahras
ar.OpenInFahras=فتح في فهرس

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[InstallDelete]
; Replace the runtime as a whole so files dropped by a newer GTK do not linger.
Type: filesandordirs; Name: "{app}\bin"; Check: FahrasInstalledHere
Type: filesandordirs; Name: "{app}\lib"; Check: FahrasInstalledHere
Type: filesandordirs; Name: "{app}\share"; Check: FahrasInstalledHere

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Registry]
; "Open in Fahras" on folders, drives and a folder's empty space in Explorer.
Root: HKA; Subkey: "Software\Classes\Directory\shell\Fahras"; ValueType: string; ValueName: ""; ValueData: "{cm:OpenInFahras}"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Directory\shell\Fahras"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\bin\Fahras.exe"
Root: HKA; Subkey: "Software\Classes\Directory\shell\Fahras\command"; ValueType: string; ValueName: ""; ValueData: """{app}\bin\Fahras.exe"" ""%1"""
Root: HKA; Subkey: "Software\Classes\Drive\shell\Fahras"; ValueType: string; ValueName: ""; ValueData: "{cm:OpenInFahras}"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Drive\shell\Fahras"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\bin\Fahras.exe"
Root: HKA; Subkey: "Software\Classes\Drive\shell\Fahras\command"; ValueType: string; ValueName: ""; ValueData: """{app}\bin\Fahras.exe"" ""%1"""
Root: HKA; Subkey: "Software\Classes\Directory\Background\shell\Fahras"; ValueType: string; ValueName: ""; ValueData: "{cm:OpenInFahras}"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Directory\Background\shell\Fahras"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\bin\Fahras.exe"
Root: HKA; Subkey: "Software\Classes\Directory\Background\shell\Fahras\command"; ValueType: string; ValueName: ""; ValueData: """{app}\bin\Fahras.exe"" ""%V"""

[Icons]
Name: "{autoprograms}\Fahras"; Filename: "{app}\bin\Fahras.exe"; WorkingDir: "{userdocs}"; IconFilename: "{app}\fahras.ico"; Comment: "{cm:FahrasComment}"
Name: "{autodesktop}\Fahras"; Filename: "{app}\bin\Fahras.exe"; WorkingDir: "{userdocs}"; IconFilename: "{app}\fahras.ico"; Comment: "{cm:FahrasComment}"; Tasks: desktopicon

[Run]
Filename: "{app}\bin\Fahras.exe"; Description: "{cm:LaunchFahras}"; Flags: nowait postinstall skipifsilent
; Fahras runs the installer with /RELAUNCH=1 when it updates itself.
Filename: "{app}\bin\Fahras.exe"; Flags: nowait runasoriginaluser; Check: RelaunchAfterUpdate

[Code]
{ Only an earlier Fahras's runtime is cleared, never another program's
  bin, lib or share folders in a folder the person picked. }
function FahrasInstalledHere: Boolean;
begin
  Result := FileExists(ExpandConstant('{app}\bin\Fahras.exe'));
end;

function RelaunchAfterUpdate: Boolean;
begin
  Result := WizardSilent and (ExpandConstant('{param:relaunch|0}') = '1');
end;

{ Settings can make Fahras open folders for this user; give them back to
  File Explorer when Fahras goes. }
procedure ReleaseFolderClass(ClassKey: String);
var
  Verb: String;
begin
  if RegQueryStringValue(HKCU, ClassKey + '\shell', '', Verb) and (Verb = 'Fahras') then
    RegDeleteValue(HKCU, ClassKey + '\shell', '');
  RegDeleteKeyIncludingSubkeys(HKCU, ClassKey + '\shell\Fahras');
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    ReleaseFolderClass('Software\Classes\Directory');
    ReleaseFolderClass('Software\Classes\Drive');
  end;
end;
