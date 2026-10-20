; Only kept so that custom installer templates written against the old layout,
; where every utility macro was reachable through this one file, keep working.
; `installer.nsi` includes the files below directly.
;
; TODO(v3): remove this file

!include "console.nsh"
!include "context.nsh"
!include "dpi.nsh"
!include "process.nsh"
!include "shell.nsh"
