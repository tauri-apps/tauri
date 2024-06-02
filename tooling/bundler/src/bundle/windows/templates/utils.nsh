; Change shell and registry context based on running
; architecture and chosen install mode.
!macro SetContext
  !if "${INSTALLMODE}" == "currentUser"
    SetShellVarContext current
  !else if "${INSTALLMODE}" == "perMachine"
    SetShellVarContext all
  !endif

  ${If} ${RunningX64}
    !if "${ARCH}" == "x64"
      SetRegView 64
    !else if "${ARCH}" == "arm64"
      SetRegView 64
    !else
      SetRegView 32
    !endif
  ${EndIf}
!macroend

; Checks whether app is running or not and prompts to kill it.
!macro CheckIfAppIsRunning
  !if "${INSTALLMODE}" == "currentUser"
    nsis_tauri_utils::FindProcessCurrentUser "${MAINBINARYNAME}.exe"
  !else
    nsis_tauri_utils::FindProcess "${MAINBINARYNAME}.exe"
  !endif
  Pop $R0
  ${If} $R0 = 0
      IfSilent kill 0
      ${IfThen} $PassiveMode != 1 ${|} MessageBox MB_OKCANCEL "$(appRunningOkKill)" IDOK kill IDCANCEL cancel ${|}
      kill:
        !if "${INSTALLMODE}" == "currentUser"
          nsis_tauri_utils::KillProcessCurrentUser "${MAINBINARYNAME}.exe"
        !else
          nsis_tauri_utils::KillProcess "${MAINBINARYNAME}.exe"
        !endif
        Pop $R0
        Sleep 500
        ${If} $R0 = 0
          Goto app_check_done
        ${Else}
          IfSilent silent ui
          silent:
            System::Call 'kernel32::AttachConsole(i -1)i.r0'
            ${If} $0 != 0
              System::Call 'kernel32::GetStdHandle(i -11)i.r0'
              System::call 'kernel32::SetConsoleTextAttribute(i r0, i 0x0004)' ; set red color
              FileWrite $0 "$(appRunning)$\n"
            ${EndIf}
            Abort
          ui:
            Abort "$(failedToKillApp)"
        ${EndIf}
      cancel:
        Abort "$(appRunning)"
  ${EndIf}
  app_check_done:
!macroend

; Sets AppUserModelId on a shortcut
!macro SetLnkAppUserModelId shortcut
  !insertmacro ComHlpr_CreateInProcInstance ${CLSID_ShellLink} ${IID_IShellLink} r0 ""
  ${If} $0 P<> 0
    ${IUnknown::QueryInterface} $0 '("${IID_IPersistFile}",.r1)'
    ${If} $1 P<> 0
      ${IPersistFile::Load} $1 '("${shortcut}", ${STGM_READWRITE})'
      ${IUnknown::QueryInterface} $0 '("${IID_IPropertyStore}",.r2)'
      ${If} $2 P<> 0
        System::Call 'Oleaut32::SysAllocString(w "${BUNDLEID}") i.r3'
        System::Call '*${SYSSTRUCT_PROPERTYKEY}(${PKEY_AppUserModel_ID})p.r4'
        System::Call '*${SYSSTRUCT_PROPVARIANT}(${VT_BSTR},,&i4 $3)p.r5'
        ${IPropertyStore::SetValue} $2 '($4,$5)'

        System::Call 'Oleaut32::SysFreeString($3)'
        System::Free $4
        System::Free $5
        ${IPropertyStore::Commit} $2 ""
        ${IUnknown::Release} $2 ""
        ${IPersistFile::Save} $1 '("${shortcut}",1)'
      ${EndIf}
      ${IUnknown::Release} $1 ""
    ${EndIf}
    ${IUnknown::Release} $0 ""
  ${EndIf}
!macroend

; Deletes jump list entries and recent destinations
!macro DeleteAppUserModelId
  !insertmacro ComHlpr_CreateInProcInstance ${CLSID_DestinationList} ${IID_ICustomDestinationList} r1 ""
  ${If} $1 P<> 0
    ${ICustomDestinationList::DeleteList} $1 '("${BUNDLEID}")'
    ${IUnknown::Release} $1 ""
  ${EndIf}
  !insertmacro ComHlpr_CreateInProcInstance ${CLSID_ApplicationDestinations} ${IID_IApplicationDestinations} r1 ""
  ${If} $1 P<> 0
    ${IApplicationDestinations::SetAppID} $1 '("${BUNDLEID}")i.r0'
    ${If} $0 >= 0
      ${IApplicationDestinations::RemoveAllDestinations} $1 ''
    ${EndIf}
    ${IUnknown::Release} $1 ""
  ${EndIf}
!macroend

; Unpins a shortcut from Start menu and Taskbar
;
; From https://stackoverflow.com/a/42816728/16993372
!macro UnpinShortcut shortcut
  !insertmacro ComHlpr_CreateInProcInstance ${CLSID_StartMenuPin} ${IID_IStartMenuPinnedList} r0 ""
  ${If} $0 P<> 0
      System::Call 'SHELL32::SHCreateItemFromParsingName(ws, p0, g "${IID_IShellItem}", *p0r1)' "${shortcut}"
      ${If} $1 P<> 0
          ${IStartMenuPinnedList::RemoveFromList} $0 '(r1)'
          ${IUnknown::Release} $1 ""
      ${EndIf}
      ${IUnknown::Release} $0 ""
  ${EndIf}
!macroend

!define /ifndef ERROR_INSUFFICIENT_BUFFER 0x7A
!define /ifndef PROCESS_CREATE_PROCESS 0x80
!define /ifndef PROC_THREAD_ATTRIBUTE_PARENT_PROCESS 0x20000
!define /ifndef CREATE_UNICODE_ENVIRONMENT 0x00000010
!define /ifndef EXTENDED_STARTUPINFO_PRESENT 0x00080000
!macro RunAsUser program args
  ; r0 hwnd
  ; r1 pid
  ; r2 process
  ; r3 size
  ; r4 attribute list
  ; r5 STARTUPINFOEX
  ; r6 PROCESS_INFORMATION
  ; r7 PROCESS_INFORMATION.hProcess
  ; r8 PROCESS_INFORMATION.hThread
  ; r9 error code

  ; Get the Shell Window handle
  System::Call 'user32::GetShellWindow() p .r0'
  ${If} $0 <> 0
    ; Get the Process ID of the Shell Window
    System::Call 'user32::GetWindowThreadProcessId(p r0, *i .r1) i .r9'
    ${If} $9 <> 0
      ; Open the Shell Process
      System::Call 'kernel32::OpenProcess(i ${PROCESS_CREATE_PROCESS}, i 0, i r1) p .r2'
      ${If} $2 <> 0
        ; Get attribute list size
        System::Call 'kernel32::InitializeProcThreadAttributeList(p 0, i 1, i 0, *i .r3) ? e'
        Pop $9
        ${If} $9 = ${ERROR_INSUFFICIENT_BUFFER}
          ; Allocate memory for attribute list
          System::Alloc $3
          Pop $4
          ; Initialize the attribute list
          System::Call 'kernel32::InitializeProcThreadAttributeList(p r4, i 1, i 0, *i r3) i .r9'
          ${If} $9 <> 0
            ; Update the attribute list with the parent process
            System::Call 'kernel32::UpdateProcThreadAttribute(p r4, i 0, i ${PROC_THREAD_ATTRIBUTE_PARENT_PROCESS}, *p r2, &i 4, p 0, p 0) i .r9'
            ${If} $9 <> 0
              System::Call 'kernel32::LocalAlloc(i 0, i 104) p .r5' ; Allocate 104 bytes for STARTUPINFOEX (68 bytes for STARTUPINFO + 36 bytes for PPROC_THREAD_ATTRIBUTE_LIST)
              System::Call 'kernel32::ZeroMemory(p r5, i 104)' ; Zero memory for STARTUPINFOEX
              System::Call '*$5(i 104, p r4)' ; Set cb to 104 (size of STARTUPINFOEX) and set lpAttributeList to r4

              ; Allocate memory for PROCESS_INFORMATION structure
              System::Call 'kernel32::LocalAlloc(i 0, i 16) p .r6' ; Allocate 16 bytes for PROCESS_INFORMATION
              System::Call 'kernel32::ZeroMemory(p r6, i 16)' ; Zero memory for PROCESS_INFORMATION

              ; Create the process
              System::Call 'kernel32::CreateProcessW(w "${program}", w "${program} ${args}", p 0, p 0, i 0, i ${CREATE_NEW_CONSOLE} | ${EXTENDED_STARTUPINFO_PRESENT}, p 0, p 0, p r5, p r6) i .r9'
              ${If} $9 <> 0
                System::Call '*$6(p .r7, p .r8, ,)'
                System::Call 'kernel32::CloseHandle(p r7)'
                System::Call 'kernel32::CloseHandle(p r8)'
              ${EndIf}
              System::Call 'kernel32::LocalFree(p r5)'
              System::Call 'kernel32::LocalFree(p r6)'
            ${EndIf}
          ${EndIf}
          ; System::Free $3
          System::Call 'kernel32::DeleteProcThreadAttributeList(p r3)'
        ${EndIf}
        System::Call 'kernel32::CloseHandle(p r2)'
      ${EndIf}
    ${EndIf}
    System::Call 'kernel32::CloseHandle(p r0)'
  ${EndIf}
!macroend
