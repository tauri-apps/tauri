---
"tauri-utils": patch:bug
---

Fixed the `app > security > headers > Permissions-Policy` configuration being sent as a header named `Permission-Policy`, which is not a real HTTP header, so the policy had no effect. The header is now correctly named `Permissions-Policy`.
