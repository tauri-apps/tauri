; Utility macros used by the installer, grouped by topic in the `utils` folder.
;
; This file only exists to include all of them at once,
; it is also the include that custom installer templates rely on.
;
; Keep it macros and variables only, with no instructions and no dependency on
; installer state, so that it stays safe to include from anywhere in a template.
; Functions built on top of these macros live in their own file, see `shortcuts.nsh`.

!include "utils\console.nsh"
!include "utils\context.nsh"
!include "utils\dpi.nsh"
!include "utils\process.nsh"
!include "utils\shell.nsh"
