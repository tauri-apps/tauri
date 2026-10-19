; Functions to create the start menu and desktop shortcuts,
; or to update them if they were created by an older version.
;
; These are functions and not macros, so including this file emits instructions
; reading `$AppStartMenuFolder`, `$WixUninstallKey`, `$UpdateMode`, `$NoShortcutMode`
; and `$OldMainBinaryName`, which is why it is included by `installer.nsi` after
; those variables are declared instead of by `utils.nsh`. NSIS only warns about
; variables it does not know yet, so including it any earlier would still compile
; but create the shortcuts in the wrong place.

; Whether an existing shortcut was migrated to the new main binary name
Var ShortcutMigrated

; Result of the `IsShortcutTarget` checks, 1 when the shortcut points at the given target
Var ShortcutTargetMatches

Function CreateOrUpdateStartMenuShortcut
  ; We used to use product name as MAINBINARYNAME
  ; migrate old shortcuts to target the new MAINBINARYNAME
  StrCpy $ShortcutMigrated 0

  !insertmacro IsShortcutTarget "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\$OldMainBinaryName"
  Pop $ShortcutTargetMatches
  ${If} $ShortcutTargetMatches = 1
    !insertmacro SetShortcutTarget "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    StrCpy $ShortcutMigrated 1
  ${EndIf}

  !insertmacro IsShortcutTarget "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\$OldMainBinaryName"
  Pop $ShortcutTargetMatches
  ${If} $ShortcutTargetMatches = 1
    !insertmacro SetShortcutTarget "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    StrCpy $ShortcutMigrated 1
  ${EndIf}

  ${If} $ShortcutMigrated = 1
    Return
  ${EndIf}

  ; Skip creating shortcut if in update mode or no shortcut mode
  ; but always create if migrating from wix
  ${If} $WixUninstallKey == ""
    ${If} $UpdateMode = 1
    ${OrIf} $NoShortcutMode = 1
      Return
    ${EndIf}
  ${EndIf}

  !if "${STARTMENUFOLDER}" != ""
    CreateDirectory "$SMPROGRAMS\$AppStartMenuFolder"
    CreateShortcut "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
  !else
    CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\${PRODUCTNAME}.lnk"
  !endif
FunctionEnd

Function CreateOrUpdateDesktopShortcut
  ; We used to use product name as MAINBINARYNAME
  ; migrate old shortcuts to target the new MAINBINARYNAME
  !insertmacro IsShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\$OldMainBinaryName"
  Pop $ShortcutTargetMatches
  ${If} $ShortcutTargetMatches = 1
    !insertmacro SetShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    Return
  ${EndIf}

  ; Skip creating shortcut if in update mode or no shortcut mode
  ; but always create if migrating from wix
  ${If} $WixUninstallKey == ""
    ${If} $UpdateMode = 1
    ${OrIf} $NoShortcutMode = 1
      Return
    ${EndIf}
  ${EndIf}

  CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
  !insertmacro SetLnkAppUserModelId "$DESKTOP\${PRODUCTNAME}.lnk"
FunctionEnd
