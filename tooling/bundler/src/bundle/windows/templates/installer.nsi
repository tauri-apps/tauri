;--------------------------------
; Header files
!include MUI2.nsh
!include FileFunc.nsh
!include x64.nsh
!include WordFunc.nsh

;--------------------------------
; definitions

!define MANUFACTURER "{{{manufacturer}}}"
!define PRODUCTNAME "{{{product_name}}}"
!define VERSION "{{{version}}}"
!define VERSIONMAJOR "{{{version_major}}}"
!define VERSIONMINOR "{{{version_minor}}}"
!define VERSIONBUILD "{{{version_build}}}"
!define INSTALLMODE "{{{installer_mode}}}"
!define LICENSE "{{{license}}}"
!define INSTALLERICON "{{{installer_icon}}}"
!define SIDEBARIMAGE "{{{sidebar_image}}}"
!define HEADERIMAGE "{{{header_image}}}"
!define MAINBINARYNAME "{{{main_binary_name}}}"
!define MAINBINARYSRCPATH "{{{main_binary_path}}}"
!define BUNDLEID "{{{bundle_id}}}"
!define OUTFILE "{{{out_file}}}"
!define ARCH "{{{arch}}}"
!define ALLOWDOWNGRADES "{{{allow_downgrades}}}"
!define APR "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCTNAME}"

;--------------------------------
; Variables

Var AppStartMenuFolder
Var ReinstallPageCheck

;--------------------------------
; General onfiguration

Name "${PRODUCTNAME}"
OutFile "${OUTFILE}"
Unicode true
SetCompressor /SOLID lzma

!if "${INSTALLMODE}" == "perMachine"
  RequestExecutionLevel heighest
  ; Set default install location
   !if ${RunningX64}
    !if  "${ARCH}" == "x64"
      InstallDir "$PROGRAMFILES64\${PRODUCTNAME}"
    !else
      InstallDir "$PROGRAMFILES\${PRODUCTNAME}"
    !endif
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

;--------------------------------
;Interface Settings

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
; because the installation page has useful info that can be used debug any issues with the installer.
!define MUI_FINISHPAGE_NOAUTOCLOSE

; Use show readme button in the finish page to create a desktop shortcut
!define MUI_FINISHPAGE_SHOWREADME
!define MUI_FINISHPAGE_SHOWREADME_TEXT "Create desktop shortcut"
!define MUI_FINISHPAGE_SHOWREADME_FUNCTION "createDesktopShortcut"

; Show run app after installation.
!define MUI_FINISHPAGE_RUN $INSTDIR\${MAINBINARYNAME}.exe

;--------------------------------
; Installer pages

; Installer pages, must be ordered as they appear
!insertmacro MUI_PAGE_WELCOME
Page custom PageReinstall PageLeaveReinstall
!if "${LICENSE}" != ""
  !insertmacro MUI_PAGE_LICENSE "${LICENSE}"
!endif
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_STARTMENU Application $AppStartMenuFolder
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

;--------------------------------
; Uninstaller pages

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

;--------------------------------
;Languages
!insertmacro MUI_LANGUAGE English

;--------------------------------
;Installer Sections

Section Webview2
  ; Check if Webview2 is already installed
  ${If} ${RunningX64}
    ReadRegStr $4 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" "pv"
  ${Else}
    ReadRegStr $4 HKLM "SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" "pv"
  ${EndIf}
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
  File "${MAINBINARYSRCPATH}"

  ; Copy resources
  {{#each resources}}
    ${GetParent} "{{this}}" $R1
    CreateDirectory "$INSTDIR\$R1"
    File /a /oname={{this}} {{@key}}
  {{/each}}

  ; Copy external binaries
  {{#each binaries}}
    File /a /oname={{this}} {{@key}}
  {{/each}}

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
  WriteRegDWORD SHCTX "${APR}" "VersionBuild" ${VERSIONBUILD}
  WriteRegDWORD SHCTX "${APR}" "NoModify" "1"
  WriteRegDWORD SHCTX "${APR}" "NoRepair" "1"
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD SHCTX "${APR}" "EstimatedSize" "$0"

  ; Create start menu shortcut
  !insertmacro MUI_STARTMENU_WRITE_BEGIN Application
    CreateDirectory "$SMPROGRAMS\$AppStartMenuFolder"
    CreateShortcut "$SMPROGRAMS\$AppStartMenuFolder\${MAINBINARYNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    ApplicationID::Set "$SMPROGRAMS\$AppStartMenuFolder\${MAINBINARYNAME}.lnk" "${BUNDLEID}"

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

;--------------------------------
;Installer Functions

Function createDesktopShortcut
  CreateShortcut "$DESKTOP\${MAINBINARYNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
  ApplicationID::Set "$DESKTOP\${MAINBINARYNAME}.lnk" "${BUNDLEID}"
FunctionEnd

Function PageReinstall
  ; Check if there is an existing installation, if not, abort the reinstall page
  ReadRegStr $R0 SHCTX "${APR}" ""
  ReadRegStr $R1 SHCTX "${APR}" "UninstallString"
  ${IfThen} "$R0$R1" == "" ${|} Abort ${|}

  ; Compare this installar version with the existing installation and modify the messages presented to the user accordingly
  StrCpy $R4 "older"
  ReadRegStr $R0 SHCTX "${APR}" "DisplayVersion"
  ${IfThen} $R0 == "" ${|} StrCpy $R4 "unknown" ${|}

  ${VersionCompare} "$\"${VERSION}$\"" $R0 $R0
  ; Reinstalling the same version
  ${If} $R0 == 0
    StrCpy $R1 "${PRODUCTNAME} ${VERSION} is already installed. Select the operation you want to perform and click Next to continue."
    StrCpy $R2 "Add/Reinstall components"
    StrCpy $R3 "Uninstall ${PRODUCTNAME}"
    !insertmacro MUI_HEADER_TEXT "Already Installed" "Choose the maintenance option to perform."
    StrCpy $R0 "2"
  ; Upgrading
  ${ElseIf} $R0 == 1
    StrCpy $R1 "An $R4 version of ${PRODUCTNAME} is installed on your system. It's recommended that you uninstall the current version before installing. Select the operation you want to perform and click Next to continue."
    StrCpy $R2 "Uninstall before installing"
    StrCpy $R3 "Do not uninstall"
    !insertmacro MUI_HEADER_TEXT "Already Installed" "Choose how you want to install ${PRODUCTNAME}."
    StrCpy $R0 "1"
  ; Downgrading
  ${ElseIf} $R0 == 2
    StrCpy $R1 "A newer version of ${PRODUCTNAME} is already installed! It is not recommended that you install an older version. If you really want to install this older version, it's better to uninstall the current version first. Select the operation you want to perform and click Next to continue."
    StrCpy $R2 "Uninstall before installing"
    !if "${ALLOWDOWNGRADES}" == "true"
    StrCpy $R3 "Do not uninstall"
    !else
    StrCpy $R3 "Do not uninstall (Downgrading without uninstall is disabled for this installer)"
    !endif
    !insertmacro MUI_HEADER_TEXT "Already Installed" "Choose how you want to install ${PRODUCTNAME}."
    StrCpy $R0 "1"
  ${Else}
    Abort
  ${EndIf}

  nsDialogs::Create 1018
  Pop $R4

  ${NSD_CreateLabel} 0 0 100% 24u $R1
  Pop $R1

  ${NSD_CreateRadioButton} 30u 50u -30u 8u $R2
  Pop $R2
  ${NSD_OnClick} $R2 PageReinstallUpdateSelection

  ${NSD_CreateRadioButton} 30u 70u -30u 8u $R3
  Pop $R3
  ; disable this radio button if downgards are not allowed
  !if "${ALLOWDOWNGRADES}" == "false"
  EnableWindow $R3 0
  !endif
  ${NSD_OnClick} $R3 PageReinstallUpdateSelection

  ${If} $ReinstallPageCheck != 2
    SendMessage $R2 ${BM_SETCHECK} ${BST_CHECKED} 0
  ${Else}
    SendMessage $R3 ${BM_SETCHECK} ${BST_CHECKED} 0
  ${EndIf}

  ${NSD_SetFocus} $R2

  nsDialogs::Show
FunctionEnd

Function PageReinstallUpdateSelection
  Pop $R1

  ${NSD_GetState} $R2 $R1

  ${If} $R1 == ${BST_CHECKED}
    StrCpy $ReinstallPageCheck 1
  ${Else}
    StrCpy $ReinstallPageCheck 2
  ${EndIf}

FunctionEnd

Function PageLeaveReinstall
  ${NSD_GetState} $R2 $R1

  ; $R0 holds whether we are reinstallign the same version or not
  ; $R0 == "1" -> different versions
  ; $R0 == "2" -> same version
  ;
  ; $R1 holds the radio buttons state. its meaning is dependant on the context

  StrCmp $R0 "1" 0 +2 ; Existing install is not the same version?
    StrCmp $R1 "1" reinst_uninstall reinst_done

  StrCmp $R1 "1" reinst_done ; Same version, skip to add/reinstall components?

  reinst_uninstall:
    ReadRegStr $R1 SHCTX "${APR}" "UninstallString"

    HideWindow

    ClearErrors
    ExecWait '$R1 _?=$INSTDIR' $0

    BringToFront

    ${IfThen} ${Errors} ${|} StrCpy $0 2 ${|} ; ExecWait failed, set fake exit code

    ${If} $0 <> 0
    ${OrIf} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.exe"
      ${If} $0 = 1 ; User aborted uninstaller?
        StrCmp $R0 "2" 0 +2 ; Is the existing install the same version?
          Quit ; ...yes, already installed, we are done
        Abort
      ${EndIf}
      MessageBox MB_ICONEXCLAMATION "Unable to uninstall!"
      Abort
    ${Else}
      StrCpy $0 $R1 1
      ${IfThen} $0 == '"' ${|} StrCpy $R1 $R1 -1 1 ${|} ; Strip quotes from UninstallString
      Delete $R1
      RMDir $INSTDIR
    ${EndIf}

  reinst_done:
FunctionEnd
