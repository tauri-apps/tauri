---
"tauri": patch:bug
---

Fix runtime-resolved capabilities (feature `dynamic-acl`) colliding with the baked ACL: `Resolved::resolve` restarts `current_scope_id` at 0, so newly-resolved `scope_id`s overlapped existing ones and command scope entries were merged into the wrong plugin's bucket. Rebase by the current max `scope_id` so each runtime-added capability stays isolated.
