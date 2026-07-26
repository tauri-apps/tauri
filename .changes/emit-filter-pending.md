---
"tauri": patch:bug
---

Fix `emit_filter`/`emit_str_filter` emitting to every target when the emit is deferred onto the pending queue (which happens when it runs while the handlers lock is held, e.g. emitting from inside an event handler). The filter is now carried through the pending queue instead of degrading into a broadcast. The filter closure now requires `Send + 'static`, matching the bound already required by `Listeners::listen`.
