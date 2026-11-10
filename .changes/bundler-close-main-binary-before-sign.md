---
"tauri-bundler": "patch:bug"
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix a Windows `signCommand` that replaces the main binary (for example by writing a signed copy and renaming it over the original) failing with `Access is denied. (os error 5)`. The bundler no longer keeps the main binary open after copying it.
