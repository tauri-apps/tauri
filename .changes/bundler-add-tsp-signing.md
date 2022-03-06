---
"tauri-bundler": patch
"cli.rs": patch
"cli.js": patch
"tauri": patch
---

Added `tsp` config option under `tauri > bundle > windows`, which enables Time-Stamp Protocol (RFC 3161) for the timestamping
server under code signing on Windows if set to `true`.