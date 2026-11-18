---
"tauri": minor:feat
"tauri-runtime": minor:feat
"tauri-runtime-wry": minor:feat
---

Added `RunEvent::Suspended` and emit `RunEvent::Resumed` on Android and iOS when the application moves to the background and back to the foreground. Unlike `WindowEvent::Suspended` and `WindowEvent::Resumed`, these events are application-wide, so they are not emitted when only one of the app's activities or scenes changes state. `RunEvent::Resumed` is no longer emitted when the event loop wakes up with `ControlFlow::Poll`.
