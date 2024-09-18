; NSIS PROCESS INFO LIBRARY - GetProcessInfo.nsh
; Version 1.1 - Mar 28th, 2011
;
; Description:
;   Gets process information.
;
; Usage example:
;   ${GetProcessInfo} 0 $0 $1 $2 $3 $4
;   DetailPrint "pid=$0 parent_pid=$1 priority=$2 process_name=$3 exe=$4"
;
; History:
;   1.1 - 28/03/2011 - Added uninstall function, include guards, file header. Fixed getting full exe path on pre vista systems. (Sergius)
;

!ifndef GETPROCESSINFO_INCLUDED
!define GETPROCESSINFO_INCLUDED

!define PROCESSINFO.TH32CS_SNAPPROCESS      2
!define PROCESSINFO.INVALID_HANDLE_VALUE    -1

!define GetProcessInfo '!insertmacro GetProcessInfo'

;@in pid_in - if 0 - get current process info
;@out pid_out - real process id (may be useful, if pid_in=0)
;@out ppid - parent process id
;@out priority
;@out name - name of process
;@out fullname - fully-qualified path of process
!macro GetProcessInfo pid_in pid_out ppid priority name fullname
    Push ${pid_in}
    !ifdef __UNINSTALL__
        Call un._GetProcessInfo
    !else
        Call _GetProcessInfo
    !endif
    ;name;pri;ppid;fname;pid;
    Pop ${name}
    Pop ${priority}
    Pop ${ppid}
    Pop ${fullname}
    Pop ${pid_out}
!macroend

!macro FUNC_GETPROCESSINFO
    Exch $R3 ;pid
    Push $0
    Push $1
    Push $2
    Push $3
    Push $4
    Push $5
    Push $R0 ;hSnapshot
    Push $R1 ;result
    Push $R9 ;PROCESSENTRY32;MODULEENTRY32 and so on
    Push $R8

    ;zero registers to waste trash, if error occurred
    StrCpy $0 ""
    StrCpy $1 ""
    StrCpy $2 ""
    StrCpy $3 ""
    StrCpy $4 ""
    StrCpy $5 ""

    IntCmp $R3 0 0 skip_pid_detection skip_pid_detection
    System::Call 'kernel32::GetCurrentProcess() i.R0'
    System::Call "Kernel32::GetProcessId(i R0) i.R3"

skip_pid_detection:
    System::Call 'Kernel32::CreateToolhelp32Snapshot(i ${PROCESSINFO.TH32CS_SNAPPROCESS},i R3) i.R0'

    IntCmp $R0 ${PROCESSINFO.INVALID_HANDLE_VALUE} end ;someting wrong

    ;$R9=PROCESSENTRY32
    ;typedef struct tagPROCESSENTRY32 {
    ;  DWORD     dwSize;
    ;  DWORD     cntUsage;
    ;  DWORD     th32ProcessID;
    ;  ULONG_PTR th32DefaultHeapID;
    ;  DWORD     th32ModuleID;
    ;  DWORD     cntThreads;
    ;  DWORD     th32ParentProcessID;
    ;  LONG      pcPriClassBase;
    ;  DWORD     dwFlags;
    ;  TCHAR     szExeFile[MAX_PATH];
    ;}PROCESSENTRY32, *PPROCESSENTRY32;
    ;dwSize=4*9+2*260

    System::Alloc 1024
    pop $R9
    System::Call "*$R9(i 556)"

    System::Call 'Kernel32::Process32FirstW(i R0, i $R9) i.R1'
    StrCmp $R1 0 end

nnext_iteration:
    System::Call "*$R9(i,i,i.R1)" ;get PID
    IntCmp $R1 $R3 exitloop

    System::Call 'Kernel32::Process32NextW(i R0, i $R9) i.R1'
    IntCmp $R1 0 0 nnext_iteration nnext_iteration

exitloop:
    ;$0 - pid
    ;$1 - threads
    ;$2 - ppid
    ;$3 - priority
    ;$4 - process name
    System::Call "*$R9(i,i,i.r0,i,i,i.r1,i.r2,i.r3,i,&w256.r4)" ; Get next module

    ;free:
    System::Free $R9
    System::Call "Kernel32::CloseToolhelp32Snapshot(i R0)"

    ;===============
    ;now get full path and commandline

    System::Call "Kernel32::OpenProcess(i 1040, i 0, i r0)i .R0"

    StrCmp $R0 0 end

    IntOp $R8 0 + 256
    System::Call "psapi::GetModuleFileNameExW(i R0,i 0,t .r5, *i $R8)i .R1"

end:
    Pop $R8
    Pop $R9
    Pop $R1
    Pop $R0
    Exch $5
    Exch 1
    Exch $4
    Exch 2
    Exch $3
    Exch 3
    Exch $2
    Exch 4
    Pop $1
    Exch 4
    Exch $0
    Exch 5
    Pop $R3
!macroend ;FUNC_GETPROCESSINFO

Function _GetProcessInfo
    !insertmacro FUNC_GETPROCESSINFO
FunctionEnd

Function un._GetProcessInfo
    !insertmacro FUNC_GETPROCESSINFO
FunctionEnd

!endif ;GETPROCESSINFO_INCLUDED
