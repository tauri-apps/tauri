---
"tauri": patch
---

Use `attohttpc` on the HTTP API by default for bundle size optimization. `reqwest` is implemented behind the `reqwest-client` feature flag.
