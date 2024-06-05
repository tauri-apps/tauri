---
"tauri": "patch:feat"
"tauri-utils": "patch:feat"
"tauri-macros": "patch:feat"
"tauri-codegen": "patch:feat"
"tauri-cli": "patch:feat"
"@tauri-apps/cli": "patch:feat"
---

add a macro `include_image` to help using images from rust api directly like this `TrayIconBuilder::new().icon(include_image!("./icons/32x32.png")).build().unwrap();`
