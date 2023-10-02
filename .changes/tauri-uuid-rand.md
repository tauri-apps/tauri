---
'tauri': 'patch:breaking'
---

This release contains a number of breaking changes to improve the consistency of tauri internals and the public facing APIs
and simplifying the types where applicable:

- Removed `EventHandler` type.
- Added `EventId` type
- Changed `Manager::listen_global` and `Window::listen` to return the new `EventId` type instead of `EventHandler`.
- Removed the return type of `Manager::once_global` and `Window::once`
- Changed `Manager::unlisten` and `Window::unlisten` to take he new `EventId` type.
- Added `tauri::scope::ScopeEventId`
- Changed `FsScope::listen` to return the new `ScopeEventId` instead of `Uuid`.
- Added `FsScope::unlisten`
-
