---
"tauri": patch:breaking
---

Remove the `responder` part of a custom invoke system now that the responder can be set directly in the `tauri::WebviewWindow::on_message` function.
