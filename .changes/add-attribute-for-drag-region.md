---
"tauri": "minor"
---

Added
attribute `data-tauri-drag-region-container`, `data-tauri-drag-region-title`, `data-tauri-drag-region-interactive`, to
enable drag on children, control maximize on double click, control is or not interactable on button/input/etc.

Description:

`data-tauri-drag-region`: mark this element as "drag region"

`data-tauri-drag-region-container`: mark the "drag region" effects its children

`data-tauri-drag-region-title`: mark the "drag region" maximize on double click (default add when
only `data-tauri-drag-region` is set)

`data-tauri-drag-region-interactive`: mark the "drag region" wouldn't prevent event to its children that interact-able,
like button, input, etc. (check by "is property 'value' exists")

Example:

```html

<div>
  <div class="title"
       data-tauri-drag-region-container="true"
       data-tauri-drag-region-title="true"
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
