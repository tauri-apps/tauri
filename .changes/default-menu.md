---
"tauri": "patch"
"tauri-runtime": "patch"
---

* **Breaking Change** `Menu::default` will now create a menu filled with default menu items and menus. Previously, it returned an empty menu and now you can do:
    ```diff
      tauri::Builder::default()
    +   .menu(tauri::Menu::default())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    ```
