---
"tauri": 'minor:changes'
"tauri-codegen": 'minor:changes'
"tauri-utils": 'minor:changes'
---

Put dynamic ACL into a feature `dynamic-acl`, this is currently enabled by default to align with the previous behaviors, you can disable it through `default-features = false` to reduce the final binary size by not including the ACL references
