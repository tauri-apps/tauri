---
tauri-bundler: "patch:bug"
---

The bundler now sets the `ARCH` env var to the current build target to prevent potential issues with `appimagetool`'s auto-detection.
