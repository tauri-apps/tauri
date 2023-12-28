---
'tauri-cli': 'patch:breaking'
'@tauri-apps/cli': 'patch:breaking'
---

Removed `TAURI_ENV_PLATFORM_TYPE` and will not be set for CLI hook commands anymore, use `TAURI_ENV_PLATFORM` instead. Also Changed value of `TAURI_ENV_PLATFORM` and `TAURI_ENV_ARCH` values to match the target triple more acuratly:

- `darwin` and `androideabi` are no longer replaced with `macos` and `android` in `TAURI_ENV_PLATFORM`.
- `i686` and `i586` are no longer replaced with `x86` in `TAURI_ENV_ARCH`.
