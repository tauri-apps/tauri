---
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix `tauri dev` not watching the app at all when its `.taurignore` contains a pattern like `/*` that matches every entry at the top level, even if other entries are re-included with `!` negations. A `.taurignore` no longer ignores the directory it is in, which matches how `.gitignore` works.
