---
"tauri-bundler": "patch:bug"
---

Update linuxdeploy and the GTK plugin used for AppImages, fixing `EGL_BAD_PARAMETER` crashes on newer Mesa and bundling on Fedora and Arch. AppImages no longer force `GDK_BACKEND=x11` and now use the native Wayland backend on Wayland sessions.
