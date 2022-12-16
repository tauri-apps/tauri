; Original file: https://github.com/fatalwall/NSIS_strExplode/blob/1c1bdd5390c2c1f9812f2e385208e9f29ad01ade/strExplode.nsh
;
;=====================================================================================
;=
;= Project Name: NSIS_DotNetVersion
;=
;= File Name: strExplode.nsh
;= File Version: 1.0 Beta
;=
;= Descritoin: NSIS Library used to split a string into an array based
;=  on a separator character
;=
;=====================================================================================
;= Copyright (C) 2018 Peter Varney - All Rights Reserved
;= You may use, distribute and modify this code under the
;= terms of the MIT license,
;=
;= You should have received a copy of the MIT license with
;= this file. If not, visit : https://github.com/fatalwall/NSIS_strExplode
;=====================================================================================
;=
;= Usage:
;=   strExplode Length Separator String
;=
;=   Length			Variable such as $0 that the resutl is returned in
;=   Separator		A single character which will be used to split the passed string
;=   String			Original String which is passed in
;=
;= Example 1 - Known count of elements returned:
;=   !include strExplode.nsh
;=   strExplode $0 '.' '4.7.1'
;=   ${If} $0 == 3
;=     Pop $1
;=     Pop $2
;=     Pop $3
;=     DetailPrint "First Part: $1"
;=     DetailPrint "Sectond Part: $2"
;=     DetailPrint "Third Part: $3"
;=   ${EndIf}
;=
;= Example 2 - Unknown count of elements returned:
;=   !include strExplode.nsh
;=   strExplode $0 '.' '4.7.1'
;=   ${do}
;=     Pop $1
;=     ${If} ${Errors}
;=       ;All Parts have been popped
;=       ClearErrors
;=       ${ExitDo}
;=     ${Else}
;=       DetailPrint "Part Value: $1"
;=     ${EndIf}
;=   ${loop}
;=
;=====================================================================================

!include 'LogicLib.nsh'

!ifndef strExplode
!define strExplode "!insertmacro strExplode"


VAR EVAL_PART
VAR PARAM_SEPARATOR
VAR PARAM_STRING
VAR EVAL_CHAR

VAR INT_RETURN

!macro  strExplode Length  Separator   String
    Push `${Separator}`
    Push `${String}`
    !ifdef __UNINSTALL__
      Call un.strExplode
    !else
      Call strExplode
    !endif
    Pop `${Length}`
!macroend

!macro Func_strExplode un
  Function ${un}strExplode
    ClearErrors

    Pop $PARAM_STRING
    Pop $PARAM_SEPARATOR
    StrCpy $EVAL_PART '' ;ensur variable is blank
    StrCpy $INT_RETURN 0 ; Initialize Counter
    ${Do}
      StrCpy $EVAL_CHAR $PARAM_STRING "" -1 ;Get Last Character
      ${If} $EVAL_CHAR == ''
        ;End of string
        IntOp $INT_RETURN $INT_RETURN + 1 ; Increment Counter
        push $EVAL_PART
        ${ExitDo}
      ${ElseIf} $EVAL_CHAR == $PARAM_SEPARATOR
        ;Seperator found. Splitting
        IntOp $INT_RETURN $INT_RETURN + 1 ; Increment Counter
        push $EVAL_PART
        StrCpy $EVAL_PART '' ; clear
      ${Else}
        ;add next character to item
        StrCpy $EVAL_PART "$EVAL_CHAR$EVAL_PART"
      ${EndIf}
      StrCpy $PARAM_STRING $PARAM_STRING -1 ;remove Last character
    ${Loop}

    ;Number of items string was split into
    push $INT_RETURN
  FunctionEnd
!macroend

!insertmacro Func_strExplode ""
!insertmacro Func_strExplode "un."

!endif
