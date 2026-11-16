; MyNoX Download Manager's portable .exe (Inno Setup 6), built by tools/installer/build.ps1.
; It unpacks MyNoX with a `data` folder into any folder or USB stick: no registry, no uninstaller.

#ifndef AppVersion
  #define AppVersion "1.0.3"
#endif
#define AppName "MyNoX Download Manager"
#define AppExe "mynox.exe"
#define Root "..\.."
#define Bin Root + "\target\release"

[Setup]
; Its own id, never the installer's: the two never meet.
AppId={{6A7D1C3E-2F84-4B9A-9E51-0C8D3B7A5F62}
AppName={#AppName} Portable
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion} Portable
AppPublisher=MoOdY69
AppPublisherURL=https://github.com/InoshMatheesha/MyNoX-Download-Manager
AppCopyright=Copyright (C) 2026 MoOdY69
DefaultDirName={userdesktop}\MyNoX Portable
DisableProgramGroupPage=yes
DirExistsWarning=no
UsePreviousAppDir=no
Uninstallable=no
CreateUninstallRegKey=no
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
WizardStyle=modern
SetupIconFile={#Root}\crates\app\assets\icons\app\mynox.ico
OutputDir={#Root}\target\dist
OutputBaseFilename=MyNoX-{#AppVersion}-portable
Compression=lzma2/ultra64
SolidCompression=yes
CloseApplications=no
VersionInfoVersion={#AppVersion}
VersionInfoCompany=MoOdY69
VersionInfoDescription={#AppName} portable
VersionInfoProductName={#AppName}
#ifdef Sign
SignTool=mynoxsign
  #define SignFlag "signonce"
#else
  #define SignFlag ""
#endif

#include "languages.iss"

[CustomMessages]
lang_en.Launch=Start {#AppName}

[Dirs]
; The `data` folder switches MyNoX to portable mode (settings and list kept here).
Name: "{app}\data"

[Files]
Source: "{#Bin}\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion {#SignFlag}
Source: "{#Bin}\mynox-nmhost.exe"; DestDir: "{app}"; Flags: ignoreversion {#SignFlag}
Source: "{#Bin}\mynox-cli.exe"; DestDir: "{app}"; Flags: ignoreversion {#SignFlag}
Source: "{#Bin}\mynox-mcp.exe"; DestDir: "{app}"; Flags: ignoreversion {#SignFlag}
Source: "{#Root}\target\extension\chromium\*"; DestDir: "{app}\extension\chromium"; Flags: ignoreversion recursesubdirs
Source: "{#Root}\target\extension\mynox-firefox.xpi"; DestDir: "{app}\extension"; Flags: ignoreversion
Source: "{#Root}\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion
Source: "{#Root}\ATTRIBUTION.md"; DestDir: "{app}"; DestName: "ATTRIBUTION.txt"; Flags: ignoreversion
Source: "{#Root}\tools\installer\portable-README.txt"; DestDir: "{app}\data"; DestName: "README.txt"; Flags: ignoreversion

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:Launch}"; Flags: nowait postinstall skipifsilent
; Update now in a portable copy runs this silently with /DIR and /relaunch=1: MyNoX starts again.
Filename: "{app}\{#AppExe}"; Parameters: "/updated"; Flags: nowait; Check: Relaunch

[Code]
const
  RunningMutex = 'MyNoX.Running';

{ /relaunch=1 on the command line (MyNoX's Update now): start MyNoX again after unpacking. }
function Relaunch: Boolean;
begin
  Result := ExpandConstant('{param:relaunch|0}') = '1';
end;

{ Ask a running MyNoX in this folder to save and quit, and wait up to 15 s for it. }
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Code, Waited: Integer;
  Exe: String;
begin
  Exe := ExpandConstant('{app}\{#AppExe}');
  if CheckForMutexes(RunningMutex) and FileExists(Exe) then
  begin
    Exec(Exe, '/exit', '', SW_HIDE, ewWaitUntilTerminated, Code);
    Waited := 0;
    while CheckForMutexes(RunningMutex) and (Waited < 15000) do
    begin
      Sleep(250);
      Waited := Waited + 250;
    end;
  end;
  Result := '';
end;
