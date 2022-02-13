---
"cli.rs": patch
---

**Breaking change:** The extra arguments passed to `tauri dev` using `-- <ARGS>...` are now propagated to the runner (defaults to cargo). To pass arguments to your binary using Cargo, you now need to run `tauri dev -- -- <ARGS-TO-YOUR-BINARY>...` (notice the double `--`).
