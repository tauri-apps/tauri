---
'tauri-utils': 'minor:enhance'
---

Improve diagnostics for invalid plugin and permission identifiers.

Adds `Error::TomlFile`, `Error::JsonFile`, and `Error::Json5File` variants that carry the file path alongside the parse error, and updates the ACL build and capability loaders to use them. The reported message now identifies which file failed to parse, instead of just the parse error.

The `Identifier` deserializer also wraps the inner error with the offending identifier string so the message reads `invalid plugin or permission identifier '<value>': ...`, surfacing the bad entry without requiring a grep through the file.

Together these turn the previous build failure (`failed to parse JSON: identifiers can only include lowercase ASCII, hyphens which are not leading or trailing, and a single colon if using a prefix at line 16 column 23`) into something self-explanatory: `failed to parse JSON file '/path/to/permissions/default.toml': invalid plugin or permission identifier 'sqlite_proxy:allow-foo': identifiers can only include lowercase ASCII, hyphens which are not leading or trailing, and a single colon if using a prefix at line 16 column 23`.
