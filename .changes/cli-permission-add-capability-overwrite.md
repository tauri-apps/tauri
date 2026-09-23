---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

`tauri permission add` (and `tauri add`) no longer overwrites an existing `capabilities/desktop.json` or `capabilities/mobile.json` when adding a desktop-only or mobile-only plugin permission and no platform-restricted capability exists; the new capability now gets an unused file name and identifier (e.g. `desktop-2.json`). An explicitly passed capability is now used even if its platforms do not match the plugin, with a warning, instead of being silently ignored.
