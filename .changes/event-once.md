---
"tauri": patch
---

Prevent "once" events from being able to be called multiple times.
* `Window::trigger(/*...*/)` is now properly `pub` instead of `pub(crate)`.
* `Manager::once_global(/*...*/)` now returns an `EventHandler`.
* `Window::once(/*...*/)` now returns an `EventHandler`.
* (internal) `event::Listeners::trigger(/*...*/)` now handles removing "once" events.
