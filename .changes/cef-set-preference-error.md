---
'tauri-runtime-cef': 'patch:bug'
---

Fixed every Chromium preference write silently failing, which made `WebviewAttributes::proxy_url` do nothing on the CEF runtime. `CefPreferenceManager::SetPreference` declares only its `value` argument optional, so CEF's generated shim returns "failed" before touching the preference service whenever the `error` out-parameter is null — which is what the Rust binding passes for `None`. The runtime now passes a real error string on both call sites and logs its contents when a write is refused.
