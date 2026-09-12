---
"tauri": major:breaking
---

The Linux GTK bindings are now selected by the new `gtk3` and `gtk4` features instead of being always on, so `tauri` can be built for either GTK version and compiles on Linux and BSD without any GTK dependency at all.

- Runtime crates enable the right one for you: `tauri-runtime-wry` enables `gtk3` and `tauri-runtime-cef` enables `gtk4`. Apps and plugins that depend on `tauri` alone and use `Window::gtk_window`, `Window::default_vbox`, `WindowBuilder::transient_for_raw` or the Linux menu integration must enable one of them explicitly - those items are now gated behind the features.
- Enabling both selects GTK 4, the same precedence `muda` and `tray-icon` use. Cargo does that whenever the dependency graph contains runtime crates that disagree on the GTK version, and such a binary can only ever run one of them, because GTK 3 and GTK 4 cannot be initialized in the same process. The GTK APIs and the Linux menu integration now fail with the new `Error::GtkVersionMismatch` under a runtime whose GTK version is not the one that was selected, instead of reinterpreting its window objects.
- The `test` feature no longer implies GTK 3, so it can be combined with `gtk4`.
