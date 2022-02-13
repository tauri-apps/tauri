---
"tauri-bundler": patch
"api": patch
---

When building Universal macOS Binaries through the virtual target `universal-apple-darwin`:

- Expect a universal binary to be created by the user
- Ensure that binary is bundled and accessed correctly at runtime
