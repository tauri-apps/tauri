---
"tauri": patch:bug
---

Mitigate page scroll jumping on every keystroke when typing RTL text with spaces in contenteditable elements on macOS WKWebView. The fix temporarily freezes ancestor scroll containers during RTL text input to prevent WebKit's native scroll-into-view from using incorrect caret geometry for wrapped RTL lines.
