; Writes a message to the console the installer was launched from.
;
; Does nothing when there is no console attached to the installer,
; which is the case when it is launched from the shell,
; so this is only useful for silent installers.

Var ConsoleAttached
Var ConsoleHandle

!macro WriteErrorToConsole message
  System::Call 'kernel32::AttachConsole(i -1) i .s'
  Pop $ConsoleAttached
  ${If} $ConsoleAttached <> 0
    System::Call 'kernel32::GetStdHandle(i -11) i .s'
    Pop $ConsoleHandle
    ; 0x0004 is FOREGROUND_RED
    System::Call 'kernel32::SetConsoleTextAttribute(i $ConsoleHandle, i 0x0004)'
    FileWrite $ConsoleHandle "${message}"
  ${EndIf}
!macroend
