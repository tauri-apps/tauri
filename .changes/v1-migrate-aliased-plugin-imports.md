---
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix `tauri migrate` generating invalid namespace imports for aliased pluginified imports from `@tauri-apps/api`.

Inputs like `import { cli as superCli } from "@tauri-apps/api"` now migrate to `import * as superCli from "@tauri-apps/plugin-cli"` instead of producing invalid ESM syntax. The migration tests also reparse migrated JS, Svelte, and Vue output so syntax regressions are caught directly.
