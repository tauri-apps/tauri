---
"tauri-utils": minor
"tauri-api": minor
"tauri": minor
---

The Tauri files are now read on the app space instead of the `tauri` create.
You need to add a `Config` struct that derives `tauri::FromTauriContext`:
```rust
#[derive(tauri::FromTauriContext)]
struct Config;

fn main() {
  tauri::AppBuilder::<tauri::flavors::Wry, Config>::new()
}
```
