---
'tauri-cli': 'patch:bug'
'@tauri-apps/cli': 'patch:bug'
---

Only consider `tauri` features from `[target.<platform>.dependencies]` tables that match the build target when bundling, so a Windows-only `tray-icon` feature no longer makes Linux bundling require appindicator.
