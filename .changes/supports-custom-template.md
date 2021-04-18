---
"tauri-bundler": minor
"cli.rs": minor
---

 Add the new `template` field in `WindowsSettings`,this field is the path to the custom template, and if this field is not `None`, the template to which this path points will be given priority.

 Note: that the incoming string is not currently being processed, so the default is a relative path, requiring start with dots: `.`.
