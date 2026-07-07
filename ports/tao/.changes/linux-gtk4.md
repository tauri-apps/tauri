---
"tao": minor
---

Port the Linux backend from GTK3 to GTK4 (`gtk4` 0.11, `glib` 0.22). Adds an optional `libadwaita` feature. Public gtk types exposed through the Unix extension traits are now their GTK4 equivalents, and the minimum supported Rust version rises to 1.92 and the minimum system GTK to 4.10 (both required by gtk4-rs 0.11 and the `Accessible` interface the window type implements).
