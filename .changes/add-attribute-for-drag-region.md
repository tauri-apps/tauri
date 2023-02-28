---
"tauri": "minor"
---

Added new drag attributes,`data-tauri-drag-region-container`, `data-tauri-drag-region-titlebar`, `data-tauri-drag-region-interactive`, to enable drag on children, control maximize on double click, control interactive elements' behavior.

Description:

`data-tauri-drag-region`: mark this element as "drag region"

`data-tauri-drag-region-container`: children are also considered drag regions

`data-tauri-drag-region-titlebar`: the "drag region" maximizes window on double click (default when only `data-tauri-drag-region` is used)

`data-tauri-drag-region-interactive`: mark the "drag region" wouldn't prevent event to its children that interact-able,
like button, input, etc. (check by "is property 'value' exists")

Example:

```html

<div>
  <div class="title"
       data-tauri-drag-region-container="true"
       data-tauri-drag-region-titlebar="true"
       data-tauri-drag-region-interactive="true"
  >
    <div>Title</div>
    <div>Some decoration</div>
    <button>close</button>
    <div data-tauri-drag-region="false" onclick="alert('clicked')">
      some element not interactive by default
    </div>
  </div>
  <div class="content"
       data-tauri-drag-region-container="true"
  >
    <div>content</div>
    <div>content</div>
  </div>
</div>
```
