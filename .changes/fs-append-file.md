---
'@tauri-apps/api': 'patch:enhance'
'tauri': 'patch:enhance'
---

Add `append` option to `FsOptions` in the `fs` JS module, used in `writeTextFile` and `writeBinaryFile`, to be able to append to existing files instead of overwriting it.
