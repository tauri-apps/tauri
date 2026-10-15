---
"tauri": minor:feat
---

Added the `Plugin::cleanup_before_exit` hook and `plugin::Builder::on_cleanup_before_exit`, invoked by `App::cleanup_before_exit` right before the process exits so plugins can release resources such as sidecar processes. Unlike `RunEvent::Exit`, it also runs on exit paths that bypass the event loop such as `AppHandle::restart` on the main thread.
