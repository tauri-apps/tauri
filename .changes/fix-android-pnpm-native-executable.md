---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

On Android, fix the generated Gradle `BuildTask.kt` invoking a non-existent `pnpm-native` command instead of `pnpm` when using a corepack-managed pnpm install, which caused `:app:rustBuild*` tasks to fail with `A problem occurred starting process 'command 'pnpm-native''`.
