---
"cli.rs": patch
---

**Breaking change:** The extra arguments passed to `tauri dev` are now propagated to the runner. To pass arguments to your binary using Cargo, now you need to run `tauri dev -- -- --arg --to --your --binary`.
