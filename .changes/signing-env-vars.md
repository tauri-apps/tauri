---
"tauri-cli": patch:enhance
"@tauri-apps/cli": patch:enhance
---

Added new environment variables for `tauri signer sign` command, to align with existing environment variables used in `tauri build`, `tauri bundle` and `tauri signer generate`
- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PATH`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

The old environment variables are deprecated and will be removed in a future release.
- `TAURI_PRIVATE_KEY`
- `TAURI_PRIVATE_KEY_PATH`
- `TAURI_PRIVATE_KEY_PASSWORD`
