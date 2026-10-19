; DPI helpers.

; Scales a value that was designed for 96 DPI to the given DPI,
; `out` must be a variable.
!macro ScaleToDpi value dpi out
  IntOp ${out} ${value} * ${dpi}
  IntOp ${out} ${out} / 96
!macroend
!define ScaleToDpi "!insertmacro ScaleToDpi"
