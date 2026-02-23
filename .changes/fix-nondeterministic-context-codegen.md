---
'tauri-utils': 'patch:bug'
---

Sort csp/plugin/header configs when generating HashMap constructors so that `generate_context!` is deterministic.

See: https://github.com/tauri-apps/tauri/issues/14978 for more information