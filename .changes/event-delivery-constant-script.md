---
"tauri": patch:perf
---

Events emitted to the webview are now delivered by fetching them from a per-webview queue instead of evaluating a new script that embeds the payload for every event. The webview keeps every distinct script it evaluates, so emitting many events with different payloads made its memory grow without bound. The page is now notified with the same script once per batch of events, which also lets a burst of events share one round trip.
