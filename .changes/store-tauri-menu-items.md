---
'tauri': 'patch:bug'
---

Fix `Menu` creation no longer takes `MenuItemOptions` with `action` directly

```ts
const menu = await Menu.new({
  items: [{
    text: "Hello", action: () => { alert('World') }
  }]
})
await menu.popup()
```

The `Hello` item should now trigger the `alert('World')` action again.
