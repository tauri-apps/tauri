---
'tauri': 'patch:fix'
---

Fix re-entrant deadlock in `fs::Scope` event listeners. The `emit` method was invoking listeners while holding the `event_listeners` mutex, which caused a deadlock when a listener callback called `once`, `listen`, `unlisten`, or any method that emits an event (such as `allow_file` or `allow_directory`). The fix collects handler references before dropping the lock, preventing re-entrant locking on the same thread.
