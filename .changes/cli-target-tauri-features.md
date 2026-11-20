---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Only consider the `tauri` features of `[target.'cfg(..)'.dependencies]` tables that match the build target when bundling, so a `tray-icon` feature enabled only for Windows no longer makes Linux bundles require and depend on appindicator.
