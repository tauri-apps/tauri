---
"tauri": patch
---

Use [`Url.join()`](https://docs.rs/url/2.2.2/url/struct.Url.html#method.join) when building webview URLs in
`WindowManager`, to handle edge cases and leading/trailing slashes in paths and urls.
