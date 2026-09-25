---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix `tauri init` and `tauri info` detecting SvelteKit projects as plain Svelte; `tauri init` suggested the wrong dev server URL and frontend dist directory.
