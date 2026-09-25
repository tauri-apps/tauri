---
"tauri": patch:bug
---

Fix capabilities added at runtime through `add_capability` (feature `dynamic-acl`) being merged into the command scopes of the build time ACL, which polluted unrelated plugin scopes and made their deserialization fail. Scope values are also no longer duplicated when a scoped permission allows more than one command.
