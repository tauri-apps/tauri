---
tauri: patch:bug
---

The `tauri` custom protocol now resolves the asset path from the request URI path instead of stripping a hardcoded `tauri://localhost` prefix, so a runtime is free to define any custom scheme URL format in `Runtime::custom_scheme_url`.
