---
tauri: 'minor:bug'
---

Fix `use-after-free` unsoundness for using `State::inner` after `Manager::unmanage`, see tauri-apps/tauri#12721 for details.
