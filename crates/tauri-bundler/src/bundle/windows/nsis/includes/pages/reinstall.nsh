; Custom page asking the user whether to reinstall or uninstall,
; only shown if a previous installation was detected.
;
; The page itself is declared in `installer.nsi` with
; `Page custom PageReinstall PageLeaveReinstall`.
;
; `installer.nsi` also calls `FindWixInstall` from `.onInit`, to compare
; against the same installation this page would when it is skipped.

; Which radio button was selected the last time the page was shown,
; 1 for the first option and 2 for the second one
Var ReinstallPageCheck

; Text of the page controls
Var ReinstallMessage
Var ReinstallFirstOptionText
Var ReinstallSecondOptionText

; Window handles of the page controls
Var ReinstallDialog
Var ReinstallLabel
Var ReinstallFirstOption
Var ReinstallSecondOption

; State of the first radio button, `${BST_CHECKED}` when the user
; selected the first option and 0 when they selected the second one
Var ReinstallSelection

; Values read while looking for a previous WiX installation
Var WixSearchIndex
Var WixSearchKey
Var WixDisplayName
Var WixPublisher
Var WixUninstallString
Var WixMsiexecOffset

; Uninstall registry key of the previous WiX installation `FindWixInstall` found,
; empty if there is none
Var WixSearchResult

; Values of our own uninstall registry key, used to detect a previous
; installation and to compare its version with the one we are installing
Var ExistingUninstallKeyValue
Var ExistingUninstallString

; Whether the previous installation is an `$(older)` or an `$(unknown)` version,
; interpolated into the `olderOrUnknownVersionInstalled` language string
Var ExistingVersionKind

; Command used to uninstall the previous installation and its exit code
Var UninstallCommand
Var UninstallExitCode

; Looks for a previous WiX installation and sets `$WixSearchResult` to its
; uninstall registry key, or to an empty string if there is none.
;
; A WiX installer stores the installation info in registry
; using a UUID and so we have to loop through all keys under
; `HKLM\${WINDOWSUNINSTKEY}`
; and check if `DisplayName` and `Publisher` keys match ${PRODUCTNAME} and ${MANUFACTURER}
;
; This has a potential issue that there maybe another installation that matches
; our ${PRODUCTNAME} and ${MANUFACTURER} but wasn't installed by our WiX installer,
; however, this should be fine since the user will have to confirm the uninstallation
; and they can chose to abort it if doesn't make sense.
Function FindWixInstall
  StrCpy $WixSearchResult ""
  StrCpy $WixSearchIndex 0
  wix_loop:
    EnumRegKey $WixSearchKey HKLM "${WINDOWSUNINSTKEY}" $WixSearchIndex
    StrCmp $WixSearchKey "" wix_loop_done ; Exit loop if there is no more keys to loop on
    IntOp $WixSearchIndex $WixSearchIndex + 1
    ReadRegStr $WixDisplayName HKLM "${WINDOWSUNINSTKEY}\$WixSearchKey" "DisplayName"
    ReadRegStr $WixPublisher HKLM "${WINDOWSUNINSTKEY}\$WixSearchKey" "Publisher"
    StrCmp "$WixDisplayName$WixPublisher" "${PRODUCTNAME}${MANUFACTURER}" 0 wix_loop
    ; Only installations that are uninstalled through `msiexec` were installed by WiX
    ReadRegStr $WixUninstallString HKLM "${WINDOWSUNINSTKEY}\$WixSearchKey" "UninstallString"
    ${StrCase} $WixUninstallString $WixUninstallString "L"
    ${StrLoc} $WixMsiexecOffset $WixUninstallString "msiexec" ">"
    StrCmp $WixMsiexecOffset 0 0 wix_loop_done
    StrCpy $WixSearchResult "${WINDOWSUNINSTKEY}\$WixSearchKey"
  wix_loop_done:
FunctionEnd

Function PageReinstall
  ; Uninstall previous WiX installation if exists
  Call FindWixInstall
  ${If} $WixSearchResult != ""
    StrCpy $WixUninstallKey "$WixSearchResult"
    Goto compare_version
  ${EndIf}

  ; Check if there is an existing installation, if not, abort the reinstall page
  ReadRegStr $ExistingUninstallKeyValue SHCTX "${UNINSTKEY}" ""
  ReadRegStr $ExistingUninstallString SHCTX "${UNINSTKEY}" "UninstallString"
  ${IfThen} "$ExistingUninstallKeyValue$ExistingUninstallString" == "" ${|} Abort ${|}

  ; Compare this installar version with the existing installation
  ; and modify the messages presented to the user accordingly
  compare_version:
  StrCpy $ExistingVersionKind "$(older)"
  ${If} $WixUninstallKey != ""
    ReadRegStr $ExistingVersion HKLM "$WixUninstallKey" "DisplayVersion"
  ${Else}
    ReadRegStr $ExistingVersion SHCTX "${UNINSTKEY}" "DisplayVersion"
  ${EndIf}
  ${IfThen} $ExistingVersion == "" ${|} StrCpy $ExistingVersionKind "$(unknown)" ${|}
  ; TODO(v3): only kept for custom language files that interpolate `$R4`
  StrCpy $R4 "$ExistingVersionKind"

  nsis_tauri_utils::SemverCompare "${VERSION}" $ExistingVersion
  Pop $VersionCompareResult
  ; Reinstalling the same version
  ${If} $VersionCompareResult = 0
    StrCpy $ReinstallMessage "$(alreadyInstalledLong)"
    StrCpy $ReinstallFirstOptionText "$(addOrReinstall)"
    StrCpy $ReinstallSecondOptionText "$(uninstallApp)"
    !insertmacro MUI_HEADER_TEXT "$(alreadyInstalled)" "$(chooseMaintenanceOption)"
  ; Upgrading
  ${ElseIf} $VersionCompareResult = 1
    StrCpy $ReinstallMessage "$(olderOrUnknownVersionInstalled)"
    StrCpy $ReinstallFirstOptionText "$(uninstallBeforeInstalling)"
    StrCpy $ReinstallSecondOptionText "$(dontUninstall)"
    !insertmacro MUI_HEADER_TEXT "$(alreadyInstalled)" "$(choowHowToInstall)"
  ; Downgrading
  ${ElseIf} $VersionCompareResult = -1
    StrCpy $ReinstallMessage "$(newerVersionInstalled)"
    StrCpy $ReinstallFirstOptionText "$(uninstallBeforeInstalling)"
    !if "${ALLOWDOWNGRADES}" == "true"
      StrCpy $ReinstallSecondOptionText "$(dontUninstall)"
    !else
      StrCpy $ReinstallSecondOptionText "$(dontUninstallDowngrade)"
    !endif
    !insertmacro MUI_HEADER_TEXT "$(alreadyInstalled)" "$(choowHowToInstall)"
  ${Else}
    Abort
  ${EndIf}

  ; Skip showing the page if passive
  ;
  ; Note that we don't call this earlier at the beginning
  ; of this function because we need to populate some variables
  ; related to current installed version if detected and whether
  ; we are downgrading or not.
  ${If} $PassiveMode = 1
    Call PageLeaveReinstall
  ${Else}
    nsDialogs::Create 1018
    Pop $ReinstallDialog
    ${IfThen} $(^RTL) = 1 ${|} nsDialogs::SetRTL $(^RTL) ${|}

    ${NSD_CreateLabel} 0 0 100% 24u $ReinstallMessage
    Pop $ReinstallLabel

    ${NSD_CreateRadioButton} 30u 50u -30u 8u $ReinstallFirstOptionText
    Pop $ReinstallFirstOption
    ${NSD_OnClick} $ReinstallFirstOption PageReinstallUpdateSelection

    ${NSD_CreateRadioButton} 30u 70u -30u 8u $ReinstallSecondOptionText
    Pop $ReinstallSecondOption
    ; Disable this radio button if downgrading and downgrades are disabled
    !if "${ALLOWDOWNGRADES}" == "false"
      ${IfThen} $VersionCompareResult = -1 ${|} EnableWindow $ReinstallSecondOption 0 ${|}
    !endif
    ${NSD_OnClick} $ReinstallSecondOption PageReinstallUpdateSelection

    ; Check the first radio button if this the first time
    ; we enter this page or if the second button wasn't
    ; selected the last time we were on this page
    ${If} $ReinstallPageCheck <> 2
      SendMessage $ReinstallFirstOption ${BM_SETCHECK} ${BST_CHECKED} 0
    ${Else}
      SendMessage $ReinstallSecondOption ${BM_SETCHECK} ${BST_CHECKED} 0
    ${EndIf}

    ${NSD_SetFocus} $ReinstallFirstOption
    nsDialogs::Show
  ${EndIf}
FunctionEnd

Function PageReinstallUpdateSelection
  ${NSD_GetState} $ReinstallFirstOption $ReinstallSelection
  ${If} $ReinstallSelection == ${BST_CHECKED}
    StrCpy $ReinstallPageCheck 1
  ${Else}
    StrCpy $ReinstallPageCheck 2
  ${EndIf}
FunctionEnd

Function PageLeaveReinstall
  ${NSD_GetState} $ReinstallFirstOption $ReinstallSelection

  ; If migrating from Wix, always uninstall
  ${If} $WixUninstallKey != ""
    Goto reinst_uninstall
  ${EndIf}

  ; In update mode, always proceeds without uninstalling
  ${If} $UpdateMode = 1
    Goto reinst_done
  ${EndIf}

  ${If} $VersionCompareResult = 0  ; Same version, proceed
    ${If} $ReinstallSelection = 1  ; User chose to add/reinstall
      Goto reinst_done
    ${Else}                        ; User chose to uninstall
      Goto reinst_uninstall
    ${EndIf}
  ${ElseIf} $VersionCompareResult = 1 ; Upgrading
    ${If} $ReinstallSelection = 1  ; User chose to uninstall
      Goto reinst_uninstall
    ${Else}
      Goto reinst_done             ; User chose NOT to uninstall
    ${EndIf}
  ${ElseIf} $VersionCompareResult = -1 ; Downgrading
    ${If} $ReinstallSelection = 1  ; User chose to uninstall
      Goto reinst_uninstall
    ${Else}
      Goto reinst_done             ; User chose NOT to uninstall
    ${EndIf}
  ${EndIf}

  reinst_uninstall:
    HideWindow
    ClearErrors

    ${If} $WixUninstallKey != ""
      ReadRegStr $UninstallCommand HKLM "$WixUninstallKey" "UninstallString"
      ExecWait '$UninstallCommand' $UninstallExitCode
    ${Else}
      ReadRegStr $PreviousInstallDir SHCTX "${MANUPRODUCTKEY}" ""
      ReadRegStr $UninstallCommand SHCTX "${UNINSTKEY}" "UninstallString"
      ${IfThen} $UpdateMode = 1 ${|} StrCpy $UninstallCommand "$UninstallCommand /UPDATE" ${|} ; append /UPDATE
      ${IfThen} $PassiveMode = 1 ${|} StrCpy $UninstallCommand "$UninstallCommand /P" ${|} ; append /P
      StrCpy $UninstallCommand "$UninstallCommand _?=$PreviousInstallDir" ; append uninstall directory
      ExecWait '$UninstallCommand' $UninstallExitCode
    ${EndIf}

    BringToFront

    ${IfThen} ${Errors} ${|} StrCpy $UninstallExitCode 2 ${|} ; ExecWait failed, set fake exit code

    ${If} $UninstallExitCode <> 0
    ${OrIf} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.exe"
      ; User cancelled wix uninstaller? return to select un/reinstall page
      ${If} $WixUninstallKey != ""
      ${AndIf} $UninstallExitCode = 1602
        Abort
      ${EndIf}

      ; User cancelled NSIS uninstaller? return to select un/reinstall page
      ${If} $UninstallExitCode = 1
        Abort
      ${EndIf}

      ; Other errors? show generic error message and return to select un/reinstall page
      MessageBox MB_ICONEXCLAMATION "$(unableToUninstall)"
      Abort
    ${EndIf}

    ; The NSIS uninstaller was run in place (`_?=`), so it cannot delete itself
    ; and the install directory, clean them up here once the uninstall succeeded
    ; https://nsis.sourceforge.io/Docs/Chapter3.html#:~:text=path%20contains%20spaces.-,3.2.3%20Examples,-installer.exe%20/NCRC
    ${If} $WixUninstallKey == ""
    ${AndIf} $PreviousInstallDir != ""
      Delete "$PreviousInstallDir\uninstall.exe"
      RMDir "$PreviousInstallDir"
    ${EndIf}
  reinst_done:
FunctionEnd
