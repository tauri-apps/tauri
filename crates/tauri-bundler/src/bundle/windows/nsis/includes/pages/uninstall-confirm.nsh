; Functions used to add a `Delete app data` checkbox
; to the uninstaller confirmation page.
;
; The page itself is declared in `installer.nsi` with
; `!insertmacro MUI_UNPAGE_CONFIRM`.

!define /ifndef WS_EX_LAYOUTRTL 0x00400000

; Window handle of the `Delete app data` checkbox and whether it was checked,
; the state is read by `Section Uninstall`
Var DeleteAppDataCheckbox
Var DeleteAppDataCheckboxState

; Window handle of the inner dialog of the confirmation page and its DPI
Var ConfirmPageDialog
Var ConfirmPageDpi

; Extended style and DPI scaled bounds of the checkbox
Var CheckboxExStyle
Var CheckboxX
Var CheckboxY
Var CheckboxWidth
Var CheckboxHeight

; Font of the uninstaller window, applied to the checkbox
Var ConfirmPageFont

Function un.ConfirmShow ; Add a `Delete app data` check box
  FindWindow $ConfirmPageDialog "#32770" "" $HWNDPARENT ; Find inner dialog
  System::Call "user32::GetDpiForWindow(p $ConfirmPageDialog) i .s"
  Pop $ConfirmPageDpi

  ${If} $(^RTL) = 1
    StrCpy $CheckboxExStyle "${__NSD_CheckBox_EXSTYLE} | ${WS_EX_LAYOUTRTL}"
    ${ScaleToDpi} 50 $ConfirmPageDpi $CheckboxX
  ${Else}
    StrCpy $CheckboxExStyle "${__NSD_CheckBox_EXSTYLE}"
    StrCpy $CheckboxX 0
  ${EndIf}
  ${ScaleToDpi} 100 $ConfirmPageDpi $CheckboxY
  ${ScaleToDpi} 400 $ConfirmPageDpi $CheckboxWidth
  ${ScaleToDpi} 25 $ConfirmPageDpi $CheckboxHeight

  System::Call 'user32::CreateWindowEx(i $CheckboxExStyle, w "${__NSD_CheckBox_CLASS}", w "$(deleteAppData)", i ${__NSD_CheckBox_STYLE}, i $CheckboxX, i $CheckboxY, i $CheckboxWidth, i $CheckboxHeight, p $ConfirmPageDialog, i0, i0, i0) i .s'
  Pop $DeleteAppDataCheckbox

  SendMessage $HWNDPARENT ${WM_GETFONT} 0 0 $ConfirmPageFont
  SendMessage $DeleteAppDataCheckbox ${WM_SETFONT} $ConfirmPageFont 1
FunctionEnd

Function un.ConfirmLeave
  SendMessage $DeleteAppDataCheckbox ${BM_GETCHECK} 0 0 $DeleteAppDataCheckboxState
FunctionEnd
