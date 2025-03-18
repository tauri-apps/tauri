---
"tauri": minor:feat
---

Added `PathResolver::file_name` to resolve file names from content URIs on Android (leverating `std::path::Path::file_name` on other platforms).
