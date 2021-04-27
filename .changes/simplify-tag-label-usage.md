---
"tauri": patch
---

Simplify usage of app event and window label types. The following functions now
accept references the `Tag` can be borrowed as. This means an `&str` can now be
accepted for functions like `Window::emit`. This is a breaking change for the
following items, which now need to take a reference. Additionally, type inference
for `&"event".into()` will no longer work, but `&"event".to_string()` will. The
solution for this is to now just pass `"event"` because `Borrow<str>` is implemented
for the default event type `String`.

* **Breaking:** `Window::emit` now accepts `Borrow` for the event.
* **Breaking:** `Window::emit_others` now accepts `Borrow` for the event
* **Breaking:** `Window::trigger` now accepts `Borrow` for the event.
* **Breaking:** `Manager::emit_all` now accepts `Borrow` for the event.
* **Breaking:** `Manager::emit_to` now accepts `Borrow` for both the event and window label.
* **Breaking:** `Manager::trigger_global` now accepts `Borrow` for the event.
* **Breaking:** `Manager::get_window` now accepts `Borrow` for the window label.
* _(internal):_ `trait tauri::runtime::tag::TagRef` helper for accepting tag references.
  Any time you want to accept a tag reference, that trait will handle requiring the reference
  to have all the necessary bounds, and generate errors when the exposed function doesn't
  set a bound like `P::Event: Borrow<E>`.
