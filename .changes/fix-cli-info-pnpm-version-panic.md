---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix `tauri info` panicking with `unexpected end of input while parsing minor version number` when a Node.js package's reported version isn't valid semver, such as one scraped from a pnpm peer-suffixed store directory. It now falls back to omitting the "(outdated, ...)" hint instead of crashing.
