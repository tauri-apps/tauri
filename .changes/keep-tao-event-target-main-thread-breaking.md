---
"tauri-runtime-wry": minor:breaking
---

`DispatcherMainThreadContext::window_target` now takes a `Weak<EventLoopWindowTarget<Message<T>>>` instead of `EventLoopWindowTarget<Message<T>>`
