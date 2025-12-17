---
"tauri": patch:minor
---

Introducing Dashmap instead of Arc<Mutex> in ChannelDataIpcQueue, improve performance when payload larger than MAX_RAW_DIRECT_EXECUTE_THRESHOLD. No user facing changes.