---
"tauri-utils": patch:bug
---

Fix `generate_context!` panicking when the configuration path has no filename component (e.g. `/`); `does_supported_file_name_exist` now returns `false` so the macro reports a descriptive "expected tauri config file" error instead.
