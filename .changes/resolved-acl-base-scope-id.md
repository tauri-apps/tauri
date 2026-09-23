---
"tauri-utils": patch:feat
---

Added `Resolved::resolve_with_base_scope_id` to resolve an ACL with command scope ids assigned after a given value, so the result can be merged into an already resolved ACL without colliding scope ids.
