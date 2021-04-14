---
"tauri": patch
---

internal refactoring of `Params` to allow for easier usage without a private trait with only 1 implementor.

`ParamsPrivate` -> `ParamsBase`
`ManagerPrivate` -> `ManagerBase`
(new) `Args`, crate only. Now implements `Params`/`ParamsBase`.
`App` and `Window` use `WindowManager` directly
