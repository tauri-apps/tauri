---
"tauri-api": patch
---

Use ``JSON.parse(String.raw`{arg}`)`` for communicating serialized JSON objects and arrays < 1 GB to the Webview from Rust.

https://github.com/GoogleChromeLabs/json-parse-benchmark
