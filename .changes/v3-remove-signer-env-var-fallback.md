---
"tauri-cli": major:breaking
"@tauri-apps/cli": major:breaking
---

`tauri signer sign` no longer reads the deprecated `TAURI_PRIVATE_KEY`, `TAURI_PRIVATE_KEY_PATH` and `TAURI_PRIVATE_KEY_PASSWORD` environment variables. Use `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PATH` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` instead.
