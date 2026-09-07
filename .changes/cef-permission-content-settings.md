---
'tauri-runtime-cef': 'patch:bug'
---

Accepting a permission prompt no longer writes an extra content setting of its own. `PermissionPromptCallback::cont(ACCEPT)` already reaches `PermissionRequestManager::Accept()` — the same path a user's click on Chrome's Allow button takes — which persists the grant, so the write was duplicative; and because it named no top-level URL it used a wildcard secondary pattern, which for the storage-access permissions Chromium scopes to an (embedded origin, top-level site) pair granted that origin storage access on *every* top-level site. That was reachable whenever a handler answered `Allow` to a permission mapping to `PermissionKind::Other`. Media capture still records its content settings, because `MediaAccessCallback::cont` persists nothing and those settings are what `navigator.permissions.query()` and `enumerateDevices()` read.
