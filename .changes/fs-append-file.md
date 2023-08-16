---
'@tauri-apps/api': 'patch:enhance'
'tauri': 'patch:enhance'
---

Add `append` option to `FsOptions`, used in `writeTextFile` and `writeBinaryFile`, to be able to append to existing files instead of overwriting it.
