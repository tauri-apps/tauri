---
"tauri": patch
---

Window and global events can now be nested inside event handlers. They will run as soon
as the event handler closure is finished in the order they were called. Previously, calling
events inside an event handler would produce a deadlock.

Note: The order that event handlers are called when triggered is still non-deterministic.
