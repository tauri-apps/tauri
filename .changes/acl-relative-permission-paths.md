---
"tauri-utils": patch:bug
---

Fix the `failed to read plugin permissions` build error when the Cargo target directory is moved after the first build, for example when it is restored from a CI cache at a different path. Permission files generated inside `OUT_DIR` are now listed relative to it.
