---
"tauri-runtime": major:breaking
"tauri-runtime-wry": major:breaking
---

The GTK types crossing the runtime boundary are now version-agnostic raw pointers, so a runtime can use a GTK version different from the one the `tauri` crate was built with:

- `WindowDispatch::gtk_window` and `WindowDispatch::default_vbox` return `*mut c_void` (`GtkApplicationWindow*` / `GtkBox*`) instead of `gtk::ApplicationWindow` / `gtk::Box`. Both are *transfer full*: the implementation hands out a strong reference (glib's `to_glib_full`) and the caller releases it (`from_glib_full`).
- `WindowBuilder::transient_for` takes the parent as `*mut c_void` (`GtkWindow*`), also *transfer full*: the implementation must release the reference, including when it does not support transient windows.
- `RawWindow::gtk_window` and `RawWindow::default_vbox` are `*mut c_void` as well, but *transfer none*: they are borrowed for the duration of the callback and must be wrapped with `from_glib_none`.
- `tauri_runtime_wry::GtkWindow` and `tauri_runtime_wry::GtkBox` are newtypes over `*mut c_void`.

`tauri-runtime` no longer depends on the `gtk` crate.

The new `tauri_runtime::gtk` module carries the GTK version a runtime binds to: runtimes call `gtk::declare_version` before creating windows so `tauri`, which picks its bindings at compile time, can detect a mismatch instead of reinterpreting a GTK object of the other version.
