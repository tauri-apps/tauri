---
"tauri": perf
"@tauri-apps/api": perf
---

Optimized event emission by passing filters by reference, reducing closure move overhead and improving memory ergonomics. No user facing changes.
