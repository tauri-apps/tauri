---
tauri: patch:bug
---

Fixed the `asset://` protocol returning malformed multi-range responses: the multipart body ended with an opening boundary instead of the closing one, the response carried both the file mime type and the `multipart/byteranges` type in two `Content-Type` headers, and the status was `200 OK` instead of `206 Partial Content`.
