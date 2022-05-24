---
"tauri-runtime-wry": patch
---

Emit `RunEvent::Exit` on `tao::event::Event::LoopDestroyed` instead of after `RunEvent::ExitRequested`.
