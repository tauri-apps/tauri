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

Track all-frame document generations for every native CEF webview and expose
opaque document tokens for final native snapshot comparisons. Missing main
frames and renderer termination revoke prior document tokens.

Include an opaque native window lifetime in webview snapshots so reparenting
and same-label window recreation do not rely on reusable labels or handles.
