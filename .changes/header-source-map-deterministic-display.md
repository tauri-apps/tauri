---
"tauri-utils": patch:bug
---

Fixed header values configured with an object in `app > security > headers` being serialized in a random order, which made the resulting header value differ between runs. The `key value` pairs are now always sorted by key, matching the ordering already used when the configuration is serialized.
