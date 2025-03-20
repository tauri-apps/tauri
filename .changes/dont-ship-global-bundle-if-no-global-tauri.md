---
tauri: 'patch:perf'
tauri-build: 'patch:perf'
tauri-plugin: 'patch:perf'
tauri-utils: 'patch:perf'
---

Don't ship global `bundle.global.js` if `app > withGlobalTauri` is set to false
