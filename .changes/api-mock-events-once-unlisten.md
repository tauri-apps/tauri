---
"@tauri-apps/api": patch:bug
---

Fix `mockIPC` with `shouldMockEvents` so `once` listeners only run once and `unlisten` removes the mocked listener. The mocked event now carries the listener `id`, and `plugin:event|unlisten` reads `eventId`.
