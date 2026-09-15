---
"@tauri-apps/api": major:breaking
---

`UnlistenFn` is now typed as `() => Promise<void>`, matching what `listen` and friends already returned at runtime. The `callback` argument of `transformCallback` is now required.
