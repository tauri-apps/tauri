---
"tauri-utils": patch:bug
---

Fix parsing a hex `Color` string panicking when it contains non-ASCII characters, for example `#€€`; it now returns an error. Surrounding whitespace is also trimmed when the string has no leading `#`.
