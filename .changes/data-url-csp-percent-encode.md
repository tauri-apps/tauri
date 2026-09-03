---
tauri: patch:bug
---

Percent-encode the HTML when a `data:` URL is re-serialized after CSP injection. A `#` anywhere in the document (a hex color, an SVG `url(#id)` reference) previously turned the rest of the markup into a URL fragment, so the webview loaded a truncated document and rendered blank. Line breaks and tabs were dropped from the document for the same reason, which could swallow the rest of an inline script after a `//` comment.
