---
'tauri': 'major:breaking'
---

Refactor resources table for scoped access:

- Added `ResourceScope` enum.
- Changed the following methods on `ResourceTable` struct to take a scope argument:

  - `ResourceTable::add`
  - `ResourceTable::add_arc`
  - `ResourceTable::add_arc_dyn`
  - `ResourceTable::get`
  - `ResourceTable::get_any`
  - `ResourceTable::replace`
  - `ResourceTable::take`
  - `ResourceTable::take_any`
  - `ResourceTable::close`

- Changed `JsImage::into_img` signature to take a new argument to specify the scope to retrieve the `Image` if it is a `ResourceId` .
