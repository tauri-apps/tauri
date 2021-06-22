# Resource example

This example demonstrates the Tauri bundle resources functionality. The example adds `src-tauri/assets/index.js` as a resource (defined on `tauri.conf.json > tauri > bundle > resources`) and executes it using `Node.js`, locating the JavaScript file using the `tauri::api::path::resolve_path` API.
