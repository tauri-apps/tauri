---
'tauri-bundler': 'patch:enhance'
---

Code sign the main binary on Windows before trying to create the WiX and NSIS bundles to always sign the executable even if no bundles are enabled.
