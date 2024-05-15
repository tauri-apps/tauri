---
"tauri": "patch:bug"
"@tauri-apps/api": "patch:bug"
---

Revert [#9706](https://github.com/tauri-apps/tauri/pull/9706) which broke compatability between `tauri` crate and the JS `@tauri-apps/api` npm package in a patch release where it should've been in a minor release.
