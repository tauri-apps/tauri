---
tauri-bundler: "patch:enhance"
tauri-cli: "patch:enhance"
"@tauri-apps/cli": "patch:enhance"
---

The bundler now pulls the latest AppImage linuxdeploy plugin instead of using the built-in one. This should remove the libfuse requirement.
