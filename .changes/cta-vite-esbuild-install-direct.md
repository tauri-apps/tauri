---
"create-tauri-app": patch
---

Work around bugs between esbuild and npm by installing directly at the end of the sequence. Also default to using the latest on all of the installs instead of npx's cache.
