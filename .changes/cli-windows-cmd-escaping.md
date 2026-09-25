---
"tauri-cli": patch:sec
"@tauri-apps/cli": patch:sec
---

On Windows, package manager commands (`npm`, `pnpm`, `yarn`, `bun`, `deno`, `node`) are now executed directly instead of through `cmd /c`, so arguments such as `@tauri-apps/plugin-fs@>=2` are escaped properly and can no longer be interpreted as shell redirections or command separators.
