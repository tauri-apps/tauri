---
'tauri-runtime-cef': 'patch:enhance'
---

Chrome's status bubble and zoom bubble are now disabled on every webview. The status bubble is the link-target strip that slides in over the bottom-left corner of the page whenever the pointer rests on a link, and the zoom bubble the popup Chrome shows on Ctrl+Plus; both are browser chrome drawn over the application's own UI, and neither belongs in a Tauri window.
