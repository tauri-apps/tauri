---
'tauri': 'patch:sec'
---

Enforce ACL checks for IPC requests from remote origins even when no `AppManifest` is configured. Previously, custom (non-plugin) commands bypassed ACL entirely without an `AppManifest`, allowing any origin to invoke them. Now, remote origins are always subject to ACL resolution, and can only reach custom commands if an explicit `remote` capability has been granted.
