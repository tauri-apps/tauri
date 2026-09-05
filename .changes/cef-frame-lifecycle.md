---
'tauri': 'minor:feat'
'tauri-runtime-cef': 'minor:feat'
---

Add CEF frame lifecycle observers for main and child frames, including native
attachment, navigation, document commit, address changes, and teardown. The
callbacks run synchronously on CEF's UI thread and preserve the existing
main-frame navigation policy.

Expose the native parent relationship, bounds, and visibility sampled for a
`with_webview` callback, independently of requested state and renderer liveness.

Provide a checked, process-wide native DevTools message ID allocator shared by
runtime evaluation, initialization scripts, and callers of the native protocol
channel. Request IDs are never reused after cancellation or browser teardown.
