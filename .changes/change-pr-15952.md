---
"tauri": patch:changes
---

Internal refactors removing an `Arc` and a `Mutex`.
Deprecate `InvokeMessage::state` and `InvokeMessage::state_ref` that should accidentally were made public.
