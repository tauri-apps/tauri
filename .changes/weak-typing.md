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
* If you were using `Params` inside a function parameter or definition, all references to it have been replaced with a
  simple runtime that defaults to `Wry`. If you are not using a custom runtime, just remove `Params` from the definition
  of functions/items that previously took it. If you are using a custom runtime, you _may_ need to pass the runtime type
  to these functions.
* If you were using custom types for `Params` (uncommon and if you don't understand you probably were not using it), all
  methods that were previously taking the custom type now takes an `Into<String>` or a `&str`. The types were already
  required to be string-able, so just make sure to convert it into a string before passing it in if this breaking change
  affects you.

`tauri-macros`:

* (internal) Added private `default_runtime` proc macro to allow us to give item definitions a custom runtime only when
  the specified feature is enabled.

`tauri-runtime`:

* See `Params` note
* Removed `Params`, `MenuId`, `Tag`, `TagRef`.
* Added `menu::{MenuHash, MenuId, MenuIdRef}` as type aliases for the internal type that menu types now use.
  * All previous menu items that had a `MenuId` generic now use the underlying `MenuId` type without a generic.
* `Runtime`, `RuntimeHandle`, and `Dispatch` have no more generic parameter on `create_window(...)` and instead use the
  `Runtime` type directly
* `Runtime::system_tray` has no more `MenuId` generic and uses the string based `SystemTray` type directly.
* (internal) `CustomMenuItem::id_value()` is now hashed on creation and exposed as the `id` field with type `MenuHash`.

`tauri-runtime-wry`:

* See `Params` note
* update menu and runtime related types to the ones changed in `tauri-runtime`.

`tauri-utils`:

* `Assets::get` signature has changed to take a `&AssetKey` instead of `impl Into<AssetKey>` to become trait object
  safe.
