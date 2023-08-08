---
'tauri-bundler': 'patch:enhance'
---

On Windows, code sign the application binaries before trying to create the WiX and NSIS bundles to always sign the executables even if no bundle types are enabled.

On Windows, code sign the sidecar binaries if they are not signed already.
