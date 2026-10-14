---
"tauri": patch:bug
---

`PluginHandle::run_mobile_plugin_async` and `PluginHandle::run_mobile_plugin` no longer panic (which aborts the process on iOS and Android) when a mobile plugin command resolves after its caller stopped waiting, for example when the async call was cancelled by a timeout. The late response is dropped instead.
