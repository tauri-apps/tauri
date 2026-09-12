---
"tauri": patch:enhance
"tauri-macros": patch:enhance
"tauri-utils": patch:enhance
---

Commands rejected by the ACL are now also logged on the host side via `log::error!`; previously only the webview's rejected promise carried the reason. `generate_handler!` prints a build-time warning when a registered command is not referenced by any permission of the crate (plugin `COMMANDS` list or hand-written permission files), which previously made the command silently unreachable.
