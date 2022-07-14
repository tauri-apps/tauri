; Imports
!include MUI2.nsh
!include FileFunc.nsh
!include x64.nsh
; ---

; Variables
!define MANUFACTURER "{{{manufacturer}}}"
!define PRODUCTNAME "{{{product_name}}}"
!define VERSION "{{{version}}}"
!define VERSIONMAJOR "{{{version_major}}}"
!define VERSIONMINOR "{{{version_minor}}}"
!define INSTALLMODE "{{{installer_mode}}}"
!define LICENSE "{{{license}}}"
!define INSTALLERICON "{{{installer_icon}}}"
!define SIDEBARIMAGE "{{{sidebar_image}}}"
!define HEADERIMAGE "{{{header_image}}}"
!define MAINBINARYNAME "{{{main_binary_name}}}"
!define MAINBINARYPATH "{{{main_binary_path}}}"
!define APR "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCTNAME}"
Var AppStartMenuFolder ; Will be set through `MUI_PAGE_STARTMENU` page. Used to determine where to create the start menu shortcut
; ---

Unicode true
Name "${PRODUCTNAME}"
OutFile "{{{out_file}}}"

!if "${INSTALLMODE}" == "perMachine"
  RequestExecutionLevel heighest
  ; Set default install location
  !if ${RunningX64}
    InstallDir "$PROGRAMFILES64\${PRODUCTNAME}"
  !else
    InstallDir "$PROGRAMFILES\${PRODUCTNAME}"
  !endif
  ; Override with the previous install location if it exists
  InstallDirRegKey HKLM "Software\${MANUFACTURER}\${PRODUCTNAME}" ""
!else
  RequestExecutionLevel user
  InstallDir "$LOCALAPPDATA\${PRODUCTNAME}"
  InstallDirRegKey HKCU "Software\${MANUFACTURER}\${PRODUCTNAME}" ""
!endif

!if "${INSTALLERICON}" != ""
  !define MUI_ICON "${INSTALLERICON}"
!endif

!if "${SIDEBARIMAGE}" != ""
  !define MUI_WELCOMEFINISHPAGE_BITMAP "${SIDEBARIMAGE}"
!endif

!if "${HEADERIMAGE}" != ""
  !define MUI_HEADERIMAGE
  !define MUI_HEADERIMAGE_BITMAP  "${HEADERIMAGE}"
!endif

; Don't auto jump to finish page after installation page,
; because the instalation page has useful info that can be used debug any issues with the installer.
!define MUI_FINISHPAGE_NOAUTOCLOSE

; Use show readme button in the finish page to create a desktop shortcut
!define MUI_FINISHPAGE_SHOWREADME
!define MUI_FINISHPAGE_SHOWREADME_TEXT "Create desktop shortcut"
!define MUI_FINISHPAGE_SHOWREADME_FUNCTION "createDesktopShortcut"

; Show run app after installation.
!define MUI_FINISHPAGE_RUN $INSTDIR\Resources.exe

; Installer pages, must be ordered as they appear
!insertmacro MUI_PAGE_WELCOME
!if "${LICENSE}" != ""
  !insertmacro MUI_PAGE_LICENSE "${LICENSE}"
!endif
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_STARTMENU Application $AppStartMenuFolder
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

; Uninstaller pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

; Languages
!insertmacro MUI_LANGUAGE English

Function createDesktopShortcut
  CreateShortcut "$DESKTOP\${MAINBINARYNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
FunctionEnd

Section Webview2
  ; Check if Webview2 is already installed
  ReadRegStr $4 HKLM "SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" "pv"
  ReadRegStr $5 HKCU "SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" "pv"
  ${If} $4 == ""
  ${AndIf} $5 == ""
    Delete "$TEMP\MicrosoftEdgeWebview2Setup.exe"

    DetailPrint "Downloading Webview2 installer..."
    NScurl::http GET "https://go.microsoft.com/fwlink/p/?LinkId=2124703" "$TEMP\MicrosoftEdgeWebview2Setup.exe" /CANCEL /END
    Pop $0
    ${If} $0 == "OK"
      DetailPrint "Webview2 installer downloaded sucessfully"
    ${Else}
      DetailPrint "Error: Downloading Webview2 Failed - $0"
      Goto abort
    ${EndIf}

    DetailPrint "Installing Webview2..."
    ExecWait "$TEMP\MicrosoftEdgeWebview2Setup.exe /install" $1
    ${If} $1 == 0
      DetailPrint "Webview2 installed sucessfully"
    ${Else}
      DetailPrint "Error: Installing Webview2 Failed with exit code $1"
      Goto abort
    ${EndIf}
  ${EndIf}

  Goto done

  abort:
    Abort "Failed to install Webview2. The app can't run without it. Try restarting the installer"
  done:
SectionEnd

Section Install
  SetOutPath $INSTDIR

  ; Main executable
  File "${MAINBINARYPATH}"

  ; Copy resources and external binaries

  ; Create uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Save $INSTDIR in registry for future installations
  WriteRegStr SHCTX "Software\${MANUFACTURER}\${PRODUCTNAME}" "" $INSTDIR

  ; Registry information for add/remove programs
  WriteRegStr SHCTX "${APR}" "DisplayName" "${PRODUCTNAME}"
  WriteRegStr SHCTX "${APR}" "DisplayIcon" "$\"$INSTDIR\Resources.exe$\""
  WriteRegStr SHCTX "${APR}" "DisplayVersion" "$\"${VERSION}$\""
  WriteRegStr SHCTX "${APR}" "Publisher" "${MANUFACTURER}"
  WriteRegStr SHCTX "${APR}" "InstallLocation" "$\"$INSTDIR$\""
  WriteRegStr SHCTX "${APR}" "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
  WriteRegDWORD SHCTX "${APR}" "VersionMajor" ${VERSIONMAJOR}
  WriteRegDWORD SHCTX "${APR}" "VersionMinor" ${VERSIONMINOR}
  WriteRegDWORD SHCTX "${APR}" "NoModify" "1"
  WriteRegDWORD SHCTX "${APR}" "NoRepair" "1"
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD SHCTX "${APR}" "EstimatedSize" "$0"

  ; Create start menu shortcut
  !insertmacro MUI_STARTMENU_WRITE_BEGIN Application
    CreateDirectory "$SMPROGRAMS\$AppStartMenuFolder"
    CreateShortcut "$SMPROGRAMS\$AppStartMenuFolder\${MAINBINARYNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
  !insertmacro MUI_STARTMENU_WRITE_END

SectionEnd

Section Uninstall
  ; Delete the app directory and its content from disk
  RMDir /r "$INSTDIR"

  ; Remove start menu and desktop shortcuts
  !insertmacro MUI_STARTMENU_GETFOLDER Application $AppStartMenuFolder
  RMDir /r "$SMPROGRAMS\$AppStartMenuFolder"
  Delete "$DESKTOP\${MAINBINARYNAME}.lnk"

  ; Remove registry information for add/remove programs
  DeleteRegKey SHCTX "${APR}"
SectionEnd
