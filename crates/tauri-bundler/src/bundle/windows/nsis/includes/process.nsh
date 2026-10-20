; Checks whether the app is running or not and prompts to kill it.
;
; The restart manager reports every process that has the executable open,
; including processes running as a different user.

!define /ifndef ERROR_MORE_DATA 234

; Messages shown to the user, with the `{{product_name}}`
; placeholder of the language strings already replaced
Var AppRunningMessage
Var AppRunningPrompt
Var FailedToKillAppMessage

; Handle of the restart manager session, empty if it could not be started
Var RestartManagerSession

!macro CheckIfAppIsRunning executablePath productName
  !define UniqueID ${__LINE__}

  ; Replace {{product_name}} placeholder in the messages with the passed product name
  nsis_tauri_utils::StrReplace "$(appRunning)" "{{product_name}}" "${productName}"
  Pop $AppRunningMessage
  nsis_tauri_utils::StrReplace "$(appRunningOkKill)" "{{product_name}}" "${productName}"
  Pop $AppRunningPrompt
  nsis_tauri_utils::StrReplace "$(failedToKillApp)" "{{product_name}}" "${productName}"
  Pop $FailedToKillAppMessage

  !insertmacro RestartManager_StartSession $RestartManagerSession
  ${If} $RestartManagerSession != ""
    ; The restart manager results have to stay in the registers,
    ; the System plugin can only write its output to one of them.
    ; `RestartManager_RegisterFile` writes its result to `$0`
    !insertmacro RestartManager_RegisterFile $RestartManagerSession "${executablePath}"
    ${If} $0 = 0
      System::Call 'RSTRTMGR::RmGetList(p $RestartManagerSession, *i .r1, *i .r2, p 0, *i .r3) i .r0'
      ; ERROR_SUCCESS means no running processes, ERROR_MORE_DATA means there're some
      ${If} $0 = ${ERROR_MORE_DATA}
        IfSilent kill_${UniqueID} 0
        ${IfThen} $PassiveMode != 1 ${|} MessageBox MB_OKCANCEL $AppRunningPrompt IDOK kill_${UniqueID} IDCANCEL cancel_${UniqueID} ${|}
        kill_${UniqueID}:
          ; Call manually here to get the result
          ; !insertmacro RestartManager_Shutdown $RestartManagerSession
          System::Call 'RSTRTMGR::RmShutdown(p $RestartManagerSession, i ${RmForceShutdown}, p 0) i .r0'
          ${If} $0 = 0
            Goto app_check_done_${UniqueID}
          ${Else}
            IfSilent silent_${UniqueID} ui_${UniqueID}
            silent_${UniqueID}:
              !insertmacro WriteErrorToConsole "$AppRunningMessage$\n"
              !insertmacro RestartManager_EndSession $RestartManagerSession
              Abort
            ui_${UniqueID}:
              !insertmacro RestartManager_EndSession $RestartManagerSession
              Abort $FailedToKillAppMessage
          ${EndIf}
        cancel_${UniqueID}:
          !insertmacro RestartManager_EndSession $RestartManagerSession
          Abort $AppRunningMessage
        app_check_done_${UniqueID}:
      ${EndIf}
    ${EndIf}
    !insertmacro RestartManager_EndSession $RestartManagerSession
  ${EndIf}

  !undef UniqueID
!macroend
