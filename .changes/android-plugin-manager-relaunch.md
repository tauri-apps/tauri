---
tauri: patch:bug
---

On Android, fixed a crash when an activity is destroyed while another one is already running — installing an APK over the running app is the common way to hit it. The plugin manager tried to move its activity result launchers to the surviving activity, and `registerForActivityResult` rejects that with `IllegalStateException: LifecycleOwner ... is attempting to register while current state is RESUMED`. Each activity now registers its own launchers when it is created.
