---
'tauri-build': 'patch:enhance'
---

Add `config-json` cargo feature flag (enabled by default) to. Disabling this feature flag will stop cargo from rebuilding when `tauri.con.json` changes, see [#8721](https://github.com/tauri-apps/tauri/issues/8721) for more info.
