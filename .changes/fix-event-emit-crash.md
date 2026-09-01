---
"tauri-runtime-wry": patch:bug
---

Added a bounded backpressure queue for messages that fail to be delivered to the OS event loop. When `send_event` fails (e.g., Windows `PostMessage` quota exceeded with error 1816), messages are queued and retried on the next `MainEventsCleared` iteration instead of being silently dropped. Includes event coalescing: high-frequency `EvaluateScript` messages targeting the same webview are deduplicated, keeping only the latest. Also replaced all remaining `let _ = proxy.send_event(...)` silent drops with `log::warn!` diagnostics. This prevents app crashes when `emit()` is called at high rates from worker threads, fixing #8177.
