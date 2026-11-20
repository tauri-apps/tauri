---
"tauri-bundler": "patch:bug"
"tauri-cli": "patch:bug"
"@tauri-apps/cli": "patch:bug"
---

Fix MSI bundling failing with `CNDL0104 : Not a valid source file; detail: An error occurred while parsing EntityName` when the product name, publisher, homepage or long description contains `&`, for example `"publisher": "Smith & Sons"`. These values are now XML-escaped in `main.wxs` and the WiX localization file.
