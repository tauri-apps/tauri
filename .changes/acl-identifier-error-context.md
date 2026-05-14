---
'tauri-utils': 'minor:enhance'
---

Improve diagnostics for invalid plugin and permission identifiers.

Adds `Error::TomlFile`, `Error::JsonFile`, and `Error::Json5File` variants that carry the file path alongside the parse error, and updates the ACL build and capability loaders to use them. The reported message now identifies which file failed to parse, instead of just the parse error.

Adds the offending identifier value to `ParseIdentifierError::InvalidFormat`, which becomes `InvalidFormat(String)`. The error message now reads `invalid plugin or permission identifier '<value>': ...` so the offending entry is visible without grepping the file.

Together these turn the previous build failure (`failed to parse JSON: identifiers can only include lowercase ASCII, hyphens which are not leading or trailing, and a single colon if using a prefix at line 16 column 23`) into something self-explanatory: `failed to parse JSON file '/path/to/permissions/default.toml': invalid plugin or permission identifier 'sqlite_proxy:allow-foo': identifiers can only include lowercase ASCII letters, digits, hyphens (not leading or trailing), and a single colon when using a prefix at line 16 column 23`.
