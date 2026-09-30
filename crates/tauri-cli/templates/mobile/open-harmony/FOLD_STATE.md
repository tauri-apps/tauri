# OpenHarmony fold state

The generated EntryAbility reads the device's native `display.isFoldable()` and
`display.getFoldStatus()` values (API 10+) and listens to `foldStatusChange`.
It refreshes the snapshot on foreground and unregisters on ability destruction.
No window-width heuristic, polling, or privileged WebView JavaScript bridge is used.

```typescript
import { getFoldState, onFoldStateChanged } from '@tauri-apps/api/app'
import type { FoldState } from '@tauri-apps/api/app'

function apply(state: FoldState | null) {
  // null: unsupported platform, native API unavailable, or not initialized.
  // isFoldable=false: an ordinary device, not a folded cover display.
  // Choose an application-specific layout; still respect available window space.
  console.log(state)
}

// Listen before reading, without letting a slower snapshot overwrite a change.
let changed = false
const unlisten = await onFoldStateChanged(({ payload }) => {
  changed = true
  apply(payload)
})
try {
  const initial = await getFoldState()
  if (!changed) apply(initial)
} catch (error) {
  unlisten()
  throw error
}
// Call unlisten() when the view/component is disposed.
```

The getter requires `core:app:allow-fold-state`, included in `core:app:default`.
Listening uses `core:event:allow-listen` and `core:event:allow-unlisten`.

`status` is `unknown`, `expanded`, `folded` or `halfFolded`. Unknown native enum
values stay `unknown`; this is relevant to newer multi-fold devices. The values
report the **device**, not which display hosts a particular window, hinge bounds,
window segments or the angle of each hinge. No layout change is applied by Tauri.
Unsupported platforms return `null` and do not emit native fold-state events.

Existing generated projects need the fold-state lifecycle code merged into their
EntryAbility, alongside any custom lifecycle handlers. Rebuild the Rust library
as well as ArkTS: the native entry-point macro exports `tauriUpdateFoldState`.
An older library without that export remains bootable but cannot provide states.
Do not overwrite custom EntryAbility code merely to adopt the new template.
