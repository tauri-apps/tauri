---
tauri: patch:bug
---

Fix malformed `asset://` multi-range responses: the multipart body now ends with the closing boundary, `Content-Type` is only `multipart/byteranges`, and the status is `206 Partial Content`.
