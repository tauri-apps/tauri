; from https://gist.github.com/nikku/281d0ef126dbc215dd58bfd5b3a5cd5b
; fileassoc.nsh
; File association helper macros
; Written by Saivert
;
; Improved by Nikku<https://github.com/nikku>.

!macro APP_ASSOCIATE EXT FILECLASS DESCRIPTION ICON COMMANDTEXT COMMAND APPUSERID EXE PRODUCTNAME
  ; Backup the previously associated file class
  ReadRegStr $R0 SHELL_CONTEXT "Software\Classes\.${EXT}" ""
  WriteRegStr SHELL_CONTEXT "Software\Classes\.${EXT}" "${FILECLASS}_backup" "$R0"

  WriteRegStr SHELL_CONTEXT "Software\Classes\.${EXT}" "" "${FILECLASS}"
  WriteRegStr SHELL_CONTEXT "Software\Classes\.${EXT}\OpenWithProgids" "${FILECLASS}" ""

  WriteRegStr SHELL_CONTEXT "Software\Classes\${FILECLASS}" "" `${DESCRIPTION}`
  WriteRegStr SHELL_CONTEXT "Software\Classes\${FILECLASS}" "AppUserModelID" `${APPUSERID}`
  WriteRegStr SHELL_CONTEXT "Software\Classes\${FILECLASS}\DefaultIcon" "" `${ICON}`
  WriteRegStr SHELL_CONTEXT "Software\Classes\${FILECLASS}\shell" "" "open"
  WriteRegStr SHELL_CONTEXT "Software\Classes\${FILECLASS}\shell\open" "" `${COMMANDTEXT}`
  WriteRegStr SHELL_CONTEXT "Software\Classes\${FILECLASS}\shell\open\command" "" `${COMMAND}`

  WriteRegStr SHELL_CONTEXT "Software\Classes\Applications\${EXE}\SupportedTypes" ".${EXT}" ""
  WriteRegStr HKLM "Software\Clients\Media\${PRODUCTNAME}\Capabilities\FileAssociations" ".${EXT}" "${FILECLASS}"
!macroend

!macro APP_ASSOCIATE_INTO_APPLICATIONS EXE ICON COMMAND
  WriteRegStr SHELL_CONTEXT "Software\Classes\Applications\${EXE}\DefaultIcon" "" `${ICON}`
  WriteRegStr SHELL_CONTEXT "Software\Classes\Applications\${EXE}\shell\open\command" "" `${COMMAND}`
!macroend

!macro APP_ASSOCIATE_REMOVE_FROM_APPLICATIONS EXE
  DeleteRegKey SHELL_CONTEXT "Software\Classes\Applications\${EXE}"
!macroend

!macro APP_UNASSOCIATE EXT FILECLASS PRODUCTNAME
  ; Restore the previously backedup associated file class
  ReadRegStr $R0 SHELL_CONTEXT "Software\Classes\.${EXT}" `${FILECLASS}_backup`
  WriteRegStr SHELL_CONTEXT "Software\Classes\.${EXT}" "" "$R0"

  DeleteRegValue SHELL_CONTEXT `Software\Classes\.${EXT}` `${FILECLASS}_backup`
  DeleteRegValue SHELL_CONTEXT `Software\Classes\.${EXT}\OpenWithProgids` "${FILECLASS}"
  DeleteRegKey SHELL_CONTEXT `Software\Classes\${FILECLASS}`

  DeleteRegKey HKLM "Software\Clients\Media\${PRODUCTNAME}"
!macroend
