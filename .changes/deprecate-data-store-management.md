---
"tauri": minor:changes
"@tauri-apps/api": minor:changes
---

Deprecate the macOS / iOS only `AppHandle::fetch_data_store_identifiers`, `AppHandle::remove_data_store`, `fetchDataStoreIdentifiers` and `removeDataStore` in favor of `AppHandle::fetch_profile_identifiers`, `AppHandle::remove_profile`, `fetchProfileIdentifiers` and `removeProfile`.
