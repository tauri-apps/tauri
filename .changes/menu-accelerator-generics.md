---
'tauri': 'patch:breaking'
---

All menu item constructors `accelerator` argument have been changed to `Option<impl AsRef<str>>` so when providing `None` you need to specify the generic argument like `None::<&str>`.
