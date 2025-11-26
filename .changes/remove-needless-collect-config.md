---
'@tauri-apps/cli': 'patch:enhance'
'tauri-cli': 'patch:enhance'
---

Remove needless collect operations in config handling.

Eliminates unnecessary intermediate `Vec` allocations when passing config references to functions that accept slices, improving performance across dev, build, bundle, and iOS commands.
