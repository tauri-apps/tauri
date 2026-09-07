---
'tauri-runtime-cef': 'patch:enhance'
---

CEF now logs to `cef.log` inside the runtime cache directory instead of dropping a `debug.log` into whatever directory the application was launched from, and release builds log at `WARNING` rather than CEF's chattier `INFO` default. Both are configurable through the new `Cef::log_file` and `Cef::log_severity` builder methods.
