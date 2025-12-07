---
tauri-cli: patch:changes
---

Errors like `Error Failed to parse version 2 for for NPM package tauri` when there was no `package-lock.json` file present yet or when using ones like `link:./tauri` are now only logged in `--verbose` mode.
