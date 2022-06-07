---
"tauri": "patch"
"tauri-runtime": "patch"
---

* Change `Menu::default` to create a default menu filled with default menu items and menus. Previously, it returned an empty menu and now you can do:
    ```diff
      tauri::Builder::default()
    +   .menu(tauri::Menu::default())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    ```
