---
"tauri-cli": patch:bug
"@tauri-apps/cli": patch:bug
---

Fix iOS build failure when `Metal Toolchain` is installed by using explicit `$(DEVELOPER_DIR)/Toolchains/XcodeDefault.xctoolchain` path instead of `$(TOOLCHAIN_DIR)` for Swift library search paths.