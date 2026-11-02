; Build with makensis /DPAYLOAD=<built folder> /DOUTPUT=<setup.exe> /DICON=<icon.ico>.
; Per-user installation. No elevation, services, startup tasks or security changes.
Unicode true
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "FileFunc.nsh"
!include "x64.nsh"

!ifndef PAYLOAD
  !error "PAYLOAD must name the complete built application folder"
!endif
!ifndef OUTPUT
  !error "OUTPUT must name the installer executable"
!endif

Name "OneClient Offline"
OutFile "${OUTPUT}"
InstallDir "$LOCALAPPDATA\Programs\OneClient Offline"
InstallDirRegKey HKCU "Software\OneClientOffline" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma
SetCompressorDictSize 16
ShowInstDetails show
ShowUninstDetails show
BrandingText "OneClient Offline - unofficial community build"
VIProductVersion "2.9.0.4"
VIAddVersionKey "ProductName" "OneClient Offline"
VIAddVersionKey "FileDescription" "OneClient Offline per-user installer"
VIAddVersionKey "FileVersion" "2.9.0.4"
VIAddVersionKey "LegalCopyright" "Upstream OneClient contributors; modified community build"
!ifdef ICON
  Icon "${ICON}"
  UninstallIcon "${ICON}"
!endif

!define MUI_ABORTWARNING
!define MUI_WELCOMEPAGE_TEXT "Install the unofficial OneClient Offline launcher for your Windows account.$\r$\n$\r$\nSupports username-only offline accounts and optional local PNG skins.$\r$\n$\r$\nLocal in-game skins need a compatible CustomSkinLoader mod. Online-mode servers still require Microsoft authentication."
!define MUI_FINISHPAGE_RUN "$INSTDIR\Start OneClient.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Open OneClient Offline"
!define MUI_FINISHPAGE_RUN_NOTCHECKED
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "${PAYLOAD}\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Var Portable

Function .onInit
  SetShellVarContext current
  SetRegView 64
  StrCpy $Portable "0"
  ${GetParameters} $0
  ClearErrors
  ${GetOptions} $0 "/PORTABLE" $1
  ${IfNot} ${Errors}
    StrCpy $Portable "1"
  ${EndIf}
  ${IfNot} ${RunningX64}
    MessageBox MB_ICONSTOP "This OneClient build requires 64-bit Windows."
    Abort
  ${EndIf}
FunctionEnd

Function CheckClosed
  IfFileExists "$INSTDIR\oneclient_app.exe" 0 done
  ClearErrors
  FileOpen $0 "$INSTDIR\oneclient_app.exe" a
  ${If} ${Errors}
    MessageBox MB_ICONEXCLAMATION "Close OneClient Offline before installing this update."
    Abort
  ${Else}
    FileClose $0
  ${EndIf}
  done:
FunctionEnd

Section "OneClient Offline" Main
  Call CheckClosed
  SetOutPath "$INSTDIR"
  ; These are top-level program files only. Profile data is never bundled.
  File "${PAYLOAD}\Start OneClient.exe"
  File "${PAYLOAD}\oneclient_app.exe"
  File "${PAYLOAD}\*.dll"
  File "${PAYLOAD}\LICENSE"
  File "${PAYLOAD}\ATTRIBUTION.md"
  File "${PAYLOAD}\START-HERE.txt"
  File "${PAYLOAD}\BUILD-INFO.txt"
  File "${PAYLOAD}\OFFLINE-MODE.md"
  File "${PAYLOAD}\oneclient-offline-build.txt"
  WriteUninstaller "$INSTDIR\Uninstall OneClient Offline.exe"
  ${If} $Portable == "1"
    ; Documented extraction/test mode uses the identical payload without shell integration.
    FileOpen $0 "$INSTDIR\installer-portable.txt" w
    FileWrite $0 "Portable extraction: no shortcuts or registry entries.$\r$\n"
    FileClose $0
  ${Else}
    Delete "$INSTDIR\installer-portable.txt"
    CreateDirectory "$SMPROGRAMS\OneClient Offline"
    CreateShortcut "$SMPROGRAMS\OneClient Offline\OneClient Offline.lnk" "$INSTDIR\Start OneClient.exe"
    CreateShortcut "$SMPROGRAMS\OneClient Offline\Uninstall.lnk" "$INSTDIR\Uninstall OneClient Offline.exe"
    CreateShortcut "$DESKTOP\OneClient Offline.lnk" "$INSTDIR\Start OneClient.exe"
    WriteRegStr HKCU "Software\OneClientOffline" "InstallDir" "$INSTDIR"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneClientOffline" "DisplayName" "OneClient Offline"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneClientOffline" "DisplayVersion" "2.9.0.4"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneClientOffline" "Publisher" "Unofficial community build"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneClientOffline" "InstallLocation" "$INSTDIR"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneClientOffline" "DisplayIcon" "$INSTDIR\Start OneClient.exe"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneClientOffline" "UninstallString" '$\"$INSTDIR\Uninstall OneClient Offline.exe$\"'
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneClientOffline" "QuietUninstallString" '$\"$INSTDIR\Uninstall OneClient Offline.exe$\" /S'
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneClientOffline" "NoModify" 1
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneClientOffline" "NoRepair" 1
  ${EndIf}
SectionEnd

Function un.onInit
  SetShellVarContext current
  SetRegView 64
  StrCpy $Portable "0"
  IfFileExists "$INSTDIR\installer-portable.txt" 0 +2
    StrCpy $Portable "1"
  IfFileExists "$INSTDIR\oneclient_app.exe" 0 done
  ClearErrors
  FileOpen $0 "$INSTDIR\oneclient_app.exe" a
  ${If} ${Errors}
    MessageBox MB_ICONEXCLAMATION "Close OneClient Offline before uninstalling it."
    Abort
  ${Else}
    FileClose $0
  ${EndIf}
  done:
FunctionEnd

Section "Uninstall"
  ; Keep data/ (accounts, skins, games and settings), including on reinstall.
  Delete "$INSTDIR\Start OneClient.exe"
  Delete "$INSTDIR\oneclient_app.exe"
  Delete "$INSTDIR\concrt140.dll"
  Delete "$INSTDIR\msvcp140.dll"
  Delete "$INSTDIR\msvcp140_1.dll"
  Delete "$INSTDIR\msvcp140_2.dll"
  Delete "$INSTDIR\msvcp140_atomic_wait.dll"
  Delete "$INSTDIR\msvcp140_codecvt_ids.dll"
  Delete "$INSTDIR\vccorlib140.dll"
  Delete "$INSTDIR\vcruntime140.dll"
  Delete "$INSTDIR\vcruntime140_1.dll"
  Delete "$INSTDIR\vcruntime140_threads.dll"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\ATTRIBUTION.md"
  Delete "$INSTDIR\START-HERE.txt"
  Delete "$INSTDIR\BUILD-INFO.txt"
  Delete "$INSTDIR\OFFLINE-MODE.md"
  Delete "$INSTDIR\oneclient-offline-build.txt"
  Delete "$INSTDIR\installer-portable.txt"
  Delete "$INSTDIR\Uninstall OneClient Offline.exe"
  ${If} $Portable != "1"
    Delete "$DESKTOP\OneClient Offline.lnk"
    Delete "$SMPROGRAMS\OneClient Offline\OneClient Offline.lnk"
    Delete "$SMPROGRAMS\OneClient Offline\Uninstall.lnk"
    RMDir "$SMPROGRAMS\OneClient Offline"
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\OneClientOffline"
    DeleteRegKey HKCU "Software\OneClientOffline"
  ${EndIf}
  RMDir "$INSTDIR"
SectionEnd
