---
"tauri-runtime": major:breaking
---

`RuntimeSpecificInitAttrs` was renamed to `RuntimeInitAttrs` and now selects the runtime it belongs to:

- The trait is generic over the user event type and has a `type Runtime: Runtime<T, RuntimeInitAttrs = Self>` associated type, so the attributes alone identify the runtime.
- The implementation for `()` was removed, every runtime must define its own attributes type.
- Runtime crates must implement `From<Self>` for `dynamic::DynRuntimeInitAttrs` so their attributes can be passed to the type-erased builder.
- `Runtime::WindowOpener` and `window::WindowBuilderBase` now require `'static`, so they can be type-erased.
