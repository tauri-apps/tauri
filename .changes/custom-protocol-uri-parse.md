---
"tauri": minor
---

The custom protocol now validates the request URI. This has implications when using the `asset` protocol without the `convertFileSrc` helper, the URL must now use the `asset://localhost/$filePath` format.
