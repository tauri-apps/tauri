---
'tauri-bundler': 'major:breaking'
---

Changed resources directory location in `deb` and `rpm` to `/usr/lib/<product_name>` instead of `/usr/lib/<main_binary_name>`. For tauri v1 users, the path is unchanged as `product_name` and `main_binary_name` used the same value.
