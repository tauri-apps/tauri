---
"tauri": major:breaking
---

The Linux GTK bindings are now selected by the new `gtk3` and `gtk4` features instead of being always on, so `tauri` can be built for either GTK version and compiles on Linux and BSD without any GTK dependency at all.

- Runtime crates enable the right one for you: `tauri-runtime-wry` enables `gtk3` and `tauri-runtime-cef` enables `gtk4`. Apps and plugins that depend on `tauri` alone and use `Window::gtk_window`, `Window::default_vbox`, `WindowBuilder::transient_for_raw` or the Linux menu integration must enable one of them explicitly - those items are now gated behind the features.
- `gtk3` and `gtk4` are mutually exclusive on Linux and BSD and enabling both is a compile error: GTK 3 and GTK 4 cannot be initialized in the same process, so a single binary cannot link a GTK3 runtime and a GTK4 runtime. `--all-features` enables both and is therefore not usable on those targets; the new `__all-features-except-gtk` feature covers the rest of the feature surface for CI.
- The `test` feature no longer implies GTK 3, so it can be combined with `gtk4`.
