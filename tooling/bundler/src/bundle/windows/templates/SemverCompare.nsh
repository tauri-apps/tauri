!include "LogicLib.nsh"
!include "strExplode.nsh"

!macro SemverCompare Ver1 Ver2 return_var
    Push `${Ver2}`
    Push `${Ver1}`
    Call Func_SemverCompare
    Pop ${return_var}
!macroend

Function Func_SemverCompare
    Var /GLOBAL VER1
    Var /GLOBAL VER2
    Var /GLOBAL V1
    Var /GLOBAL V2
    Var /GLOBAL V3
    Var /GLOBAL V4
    Var /GLOBAL V5
    Var /GLOBAL VR1
    Var /GLOBAL VR2
    Var /GLOBAL VR3
    Var /GLOBAL VR4
    Var /GLOBAL VR5
    Var /GLOBAL RET

    StrCpy $VER1 ""
    StrCpy $VER2 ""
    StrCpy $V1 ""
    StrCpy $V2 ""
    StrCpy $V3 ""
    StrCpy $V4 ""
    StrCpy $V5 ""
    StrCpy $VR1 ""
    StrCpy $VR2 ""
    StrCpy $VR3 ""
    StrCpy $VR4 ""
    StrCpy $VR5 ""
    StrCpy $RET ""

    Pop $VER1
    Pop $VER2

    !insertmacro strExplode $0 '.' $VER1
    Pop $V1
    Pop $V2
    Pop $V3
    Pop $V5
    !insertmacro strExplode $0 '-' $V3
    Pop $V3
    Pop $V4

    !insertmacro strExplode $0 '.' $VER2
    Pop $VR1
    Pop $VR2
    Pop $VR3
    Pop $VR5
    !insertmacro strExplode $0 '-' $VR3
    Pop $VR3
    Pop $VR4

    ${If} $V1 > $VR1
        Goto higher
    ${EndIf}

    ${If} $V1 < $VR1
        Goto lower
    ${EndIf}

    ${If} $V2 > $VR2
        Goto higher
    ${EndIf}

    ${If} $V2 < $VR2
        Goto lower
    ${EndIf}

    ${If} $V3 > $VR3
        Goto higher
    ${EndIf}

    ${If} $V3 < $VR3
        Goto lower
    ${EndIf}

    ${If} $V4 == "beta"
        ${If} $VR4 == ""
            Goto lower
        ${ElseIf} $VR4 == "alpha"
            Goto higher
        ${EndIf}
    ${EndIf}
    ${If} $V4 == "alpha"
        ${If} $VR4 == ""
            Goto lower
        ${ElseIf} $VR4 == "beta"
            Goto lower
        ${EndIf}
    ${EndIf}
    ${If} $V4 == ""
        ${If} $VR4 == ""
            Goto equal
        ${Else}
            Goto higher
        ${EndIf}
    ${EndIf}

    ${If} $VR4 == "beta"
        ${If} $V4 == ""
            Goto higher
        ${ElseIf} $V4 == "alpha"
            Goto lower
        ${EndIf}
    ${EndIf}
    ${If} $VR4 == "alpha"
        ${If} $V4 == ""
            Goto higher
        ${ElseIf} $V4 == "beta"
            Goto higher
        ${EndIf}
    ${EndIf}
    ${If} $VR4 == ""
        ${If} $V4 == ""
            Goto equal
        ${Else}
            Goto lower
        ${EndIf}
    ${EndIf}

    ${If} $V5 > $VR5
        Goto higher
    ${EndIf}

    ${If} $V5 < $VR5
        Goto lower
    ${EndIf}

    Goto equal

    higher:
        StrCpy $RET 1
        Goto done
    lower:
        StrCpy $RET -1
        Goto done
    equal:
        StrCpy $RET 0
        Goto done
    done:
        Push $RET
FunctionEnd