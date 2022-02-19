---
"tauri-build": patch
---

Automatically emit `cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET` with the value set on `tauri.conf.json > tauri > bundle > macos > minimumSystemVersion`.
