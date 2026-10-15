---
"tauri-runtime-cef": patch:feat
---

Added `Cef::downgrade` and `DowngradePolicy`, for an application whose release is rolled back to an older CEF. The runtime now records the Chromium version in the root cache path (`Last Version`, the breadcrumb Chrome's own downgrade manager keeps and CEF does not write) and, when a newer Chromium milestone last used the profile, either keeps it and logs a warning (`DowngradePolicy::KeepProfile`, the default and what Chrome does) or moves it aside so Chromium starts on an empty profile and deletes the old one in the background (`DowngradePolicy::ResetProfile`). A downgrade within a milestone, and a profile another running instance of the application holds, are left alone.
