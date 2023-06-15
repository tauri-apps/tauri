---
'tauri': 'patch:enhance'
'tauri-build': 'patch:enhance'
'tauri-codegen': 'patch:enhance'
'tauri-runtime': 'patch:enhance'
'tauri-runtime-wry': 'patch:enhance'
---

Unpin `time`, `ignore`, `winnow`, and `ignore` crate versions. Developers now have to pin crates if needed themselves. A list of crates that need pinning to adhere to Tauri's MSRV will be visible in Tauri's GitHub workflow: https://github.com/tauri-apps/tauri/blob/dev/.github/workflows/test-core.yml#L85.
