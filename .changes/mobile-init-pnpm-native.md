---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Detect pnpm from `npm_config_user_agent` when generating the mobile projects. pnpm's native binary (pnpm 11+) runs package scripts without setting `PNPM_PACKAGE_NAME`, and when installed through corepack the binary is named `pnpm-native`, so `tauri android init` recorded `pnpm-native` as the command for Gradle to run and the Android build failed with "A problem occurred starting process 'command 'pnpm-native''".
