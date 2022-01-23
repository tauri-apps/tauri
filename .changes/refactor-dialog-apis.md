---
"tauri": patch
---

**Breaking change:** Renamed dialog functions `pick_file`, `pick_files`, `pick_folder`, `save_file`, `ask`, `confirm` and `message` to include a `_nonblocking` suffix and add blocking variants to simplify usage when not on the main thread.
