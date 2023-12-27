---
'tauri-cli': 'patch:bug'
'@tauri-apps/cli': 'patch:bug'
---

Prevent `Invalid target triple` warnings and correctly set `TAURI_ENV_` vars when target triple contains 4 components. `darwin` and `androideabi` are no longer replaced with `macos` and `android` in `TAURI_ENV_PLATFORM`.
