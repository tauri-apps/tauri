---
"tauri": patch
"tauri-runtime": patch
"tauri-runtime-wry": patch
"tauri-macros": patch
"tauri-utils": patch
---

`Params` has been removed, along with all the associated types on it. Functions that previously accepted those
associated types now accept strings instead. Type that used a generic parameter `Params` now use `Runtime` instead. If
you use the `wry` feature, then types with a `Runtime` generic parameter should default to `Wry`, letting you omit the
explicit type and let the compiler infer it instead.

`tauri`:

* See `Params` note
* **TODO** should we change all the `&str` to `AsRef<str>`?
* **TODO** (probably list all the methods/functions that changed)

`tauri-macros`:

* (internal) Added private `default_runtime_wry` proc macro to wry as the default `Runtime` for struct/enum definitions
  if the `wry` feature is enabled.

`tauri-runtime`:

* See `Params` note
* **TODO** was there more?

`tauri-runtime-wry`:

* See `Params` note
* **TODO** was there more?

`tauri-utils`:

* `Assets::get` signature has changed to take a `&AssetKey` instead of `impl Into<AssetKey>` to become trait object
  safe.
