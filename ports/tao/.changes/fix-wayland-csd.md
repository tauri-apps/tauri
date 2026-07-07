---
'tao': patch
---

On Linux, multiple issues regarding window decoration handling for Wayland have been fixed
(#899, #1046, tauri-apps/tauri#6562, tauri-apps/tauri#13440, tauri-apps/tauri#13749, tauri-apps/tauri#14251, tauri-apps/tauri#14748).
Title bar buttons and changing of the title should now work as expected.
Furthermore, client-side decorations are no longer applied, when server-side decorations are supported.
SSD are no longer applied when decorations are disabled for a window during creation.
Toggling of SSD rendering for existing windows is however not supported at this time.
