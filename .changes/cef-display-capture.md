---
'tauri-runtime-cef': 'patch:bug'
---

A permission handler answering `Allow` no longer grants `getDisplayMedia()` a full-desktop stream. CEF builds the media stream straight from the granted permission mask, and a desktop video bit with no requested source synthesises the whole desktop and returns it with no picker at all, so an app with a blanket `.on_permission_request(|_| PermissionResponse::Allow)` was handing any page in the webview — remote content reached through a redirect included — a silent capture of the entire screen. Desktop capture is now deferred to CEF whatever the handler answers `Allow` to, which restores Chromium's desktop media picker under Chrome style and the outright refusal under Alloy style; camera and microphone grants are unchanged, and a `Deny` still refuses display capture without a prompt.
