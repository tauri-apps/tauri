---
"tauri-driver": patch:bug
---

Ensure Element Send Keys requests carry the W3C `text` field by synthesizing it from the legacy JSON Wire Protocol `value` when absent.
WebKitWebDriver 2.52+ rejects bodies without `text` ("Missing text parameter").
