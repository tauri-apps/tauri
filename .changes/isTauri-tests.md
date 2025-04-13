---
"@tauri-apps/api": patch:enhance
---

`core.isTauri` now leverages `globalThis` instead of `window` in order to be used in unit tests.
