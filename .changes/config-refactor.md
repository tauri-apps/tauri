---
"tauri-utils": minor
"tauri-api": minor
"tauri": minor
---

The Tauri files are now read on the app space instead of the `tauri` create.
Also, the `AppBuilder` `build` function now returns a Result.

You need to add a `Context` struct that derives `tauri::FromTauriContext`:
```rust
#[derive(tauri::FromTauriContext)]
struct Context;

fn main() {
  tauri::AppBuilder::<tauri::flavors::Wry, Context>::new()
    .build()
    .unwrap()
    .run();
}
```
