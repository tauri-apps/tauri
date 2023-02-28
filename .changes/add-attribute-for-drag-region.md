---
"tauri": "minor"
---

Added new drag attributes,`data-tauri-drag-region-container`, `data-tauri-drag-region-titlebar`, `data-tauri-drag-region-interactive`, to enable drag on children, control maximize on double click, control interactive elements' behavior.

Description:

`data-tauri-drag-region`: mark this element as "drag region"

`data-tauri-drag-region-container`: children are also considered drag regions

`data-tauri-drag-region-titlebar`: the "drag region" maximizes window on double click (default when only `data-tauri-drag-region` is used)

`data-tauri-drag-region-exclude`: this element doesn't trigger `drag`

Example:

```html
<div class="titlebar"
     data-tauri-drag-region-container="true"
     data-tauri-drag-region-titlebar="true"
>
  <div>Title</div>
  <div>Some decoration</div>
  <button data-tauri-drag-region-exclude>close</button>
</div>
```
