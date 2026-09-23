---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

`tauri migrate` now grants the v2 permissions that match the full v1 allowlist: `fs:allow-read-text-file` / `fs:allow-write-text-file` for `fs.readFile` / `fs.writeFile`, `shell:allow-spawn` (with the execute scope), `shell:allow-kill` and `shell:allow-stdin-write` for `shell.execute` / `shell.sidecar`, and `core:webview:allow-create-webview-window` for `window.create`. The asset protocol is now enabled whenever the v1 asset protocol allowlist was enabled, even with the default scope.
