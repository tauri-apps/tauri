---
"tauri-utils": patch:enhance
---
Uses variables for targeting different operating systems.

Fixes: [#8501](https://github.com/tauri-apps/tauri/issues/8501)

{{target}} for os ("windows", "linux", "darwin")
{{arch}} for arch ("i686", "x86_64")

eg: before parse: `"../binaries/test/{{target}}/{{arch}}/*": "resources/test/"`
    after parse: `"../binaries/test/darwin/aarch64/*": "resources/test/"`
