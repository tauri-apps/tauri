---
tauri-bundler: "patch:bug"
---

The bundler now falls back to `1` for the release in case an empty string was provided instead of using `-.` in the file name.
