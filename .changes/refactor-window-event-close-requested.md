---
"tauri": patch
---

**Breaking change:** The `WindowEvent::CloseRequested` variant now includes `label` and `signal_tx` fields to allow preventing closing the window.
