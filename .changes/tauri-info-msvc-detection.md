---
'tauri-cli': 'patch:enhance'
---

Improve Visual Studio installation detection in `tauri info` command to check for the necessary components instead of whole workloads. This also fixes the detection of minimal installations and auto-installations done by `rustup`.
