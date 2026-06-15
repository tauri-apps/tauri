---
"tauri-utils": "patch:enhance"
---

Improve diagnostics for invalid plugin and permission identifiers.

The `Identifier` deserializer now wraps the inner error with the offending identifier string so the message reads `invalid plugin or permission identifier '<value>': ...`, surfacing the bad entry without requiring a grep through the file.

The previous parse failure (`failed to parse JSON: identifiers can only include lowercase ASCII, hyphens which are not leading or trailing, and a single colon if using a prefix at line 16 column 23`) now reads `failed to parse JSON: invalid plugin or permission identifier 'sqlite_proxy:allow-foo': identifiers can only include lowercase ASCII, hyphens which are not leading or trailing, and a single colon if using a prefix at line 16 column 23`.
