---
"tauri-runtime-cef": patch:bug
---

Report an unsupported CEF API version instead of crashing. The runtime now checks the result of `cef_api_hash`: the browser process logs the version and fails with `Error::WebviewRuntimeNotInstalled`, and a helper process panics with the same message.
