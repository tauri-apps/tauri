---
"tauri-runtime-wry": minor:enhance
---

On Windows, `data_store_identifier` is now applied as a WebView2 named profile, so webviews sharing one data directory get fully isolated cookies and storage per identifier while still sharing a single browser process.
