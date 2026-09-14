---
"tauri": major:breaking
---

Removed the deprecated `Manager::unmanage` method. Wrap the state in a `Mutex<Option<T>>` and use `Option::take` if you need to drop it.
