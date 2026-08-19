---
"tauri-cli": patch:enhance
"@tauri-apps/cli": patch:enhance
---

Document the `TAURI_SIGNING_PRIVATE_KEY_PATH` environment variable and clarify that `TAURI_SIGNING_PRIVATE_KEY` accepts a string or a path for the `build` and `bundle` command but must be the literal key string for the `signer sign` command, both in `ENVIRONMENT_VARIABLES.md` and in the `signer generate` command output.
