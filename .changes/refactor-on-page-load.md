---
"tauri": patch:breaking
---

Added `WindowBuilder::on_page_load` and refactored the `Builder::on_page_load` handler to take references.
The page load hook is now triggered for load started and finished events, to determine what triggered it see `PageLoadPayload::event`.
