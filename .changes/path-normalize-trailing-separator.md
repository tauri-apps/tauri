---
tauri: patch:bug
---

Fix `path.normalize` mishandling three cases of the same normalization step: a duplicated trailing separator (`"/"` returned `"//"`), separator-terminated relative inputs turning into absolute ones (`"./"` returned `"/"`), and dropped leading `..` segments (`"../"` returned `"/"`, `"./.."` returned `"."`). A path that normalizes to nothing now returns `"."` instead of an empty string, which also covers inputs such as `"a/.."`.
