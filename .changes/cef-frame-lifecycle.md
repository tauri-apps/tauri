---
'tauri': 'minor:feat'
'tauri-runtime-cef': 'minor:feat'
---

Add CEF frame lifecycle observers for main and child frames, including native
attachment, navigation, document commit, address changes, and teardown. The
callbacks run synchronously on CEF's UI thread and preserve the existing
main-frame navigation policy.
