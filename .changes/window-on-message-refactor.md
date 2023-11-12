---
"tauri": patch:breaking
---

Changed `Window::on_message` signature to take a responder closure instead of returning the response object in order to asynchronously process the request.
