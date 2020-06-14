---
"tauri.js": patch
"tauri": patch
---

Properly reflect tauri.conf changes on `tauri dev`. `nonWebpackRequire` caches the result, so it wouldn't reflect changes on `tauri.conf.json`. Reading the file from FS and parsing it instead fixes it. Also, tauri needs to rebuild to make it work.
