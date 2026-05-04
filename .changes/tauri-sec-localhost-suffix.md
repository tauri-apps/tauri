---
tauri: patch:sec
---

Correctly handle .localhost suffix in local origins on Windows and Android to fix a security issue that made tauri think remote websites that started with a registered scheme were local websites.
For example, when registering an `app` custom protocol, Tauri would think `http://app.evil.com/` would be a local URL on Windows/Android.
