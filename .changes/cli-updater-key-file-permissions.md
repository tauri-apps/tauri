---
"tauri-cli": patch:sec
"@tauri-apps/cli": patch:sec
---

`tauri signer generate --write-keys` now creates the private key with owner-only permissions (0600) on Unix, refuses to overwrite either the private or the public key file without `--force`, and replaces existing keys atomically instead of deleting them first. `tauri signer sign` now rejects file names containing tabs or newlines (which would corrupt the signature's trusted comment), reports errors instead of panicking, and assumes an empty password instead of prompting when no password is given and stdin is not a terminal.
