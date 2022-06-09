---
"tauri": "patch"
"tauri-runtime": "patch"
---

* Add `Menu::default` which will create a menu filled with default menu items and submenus so you can do:
    ```diff
      tauri::Builder::default()
    +   .menu(tauri::Menu::default("app_name"))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    ```
