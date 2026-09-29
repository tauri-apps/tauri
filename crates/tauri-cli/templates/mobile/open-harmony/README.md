# OpenHarmony window layout

The generated `EntryAbility` enables an immersive window layout. Content can
extend behind the system bars; the bars remain visible. Keep backgrounds at the
viewport edges and inset interactive controls using the webview's safe area.

Include `viewport-fit=cover` in the frontend's existing viewport meta tag:

```html
<meta
  name="viewport"
  content="width=device-width, initial-scale=1, viewport-fit=cover"
/>
```

For layout, use CSS directly so the engine updates the padding when its safe area
changes. Apply this to the appropriate header and footer rather than adding the
same inset to both a container and its children:

```css
.app-header {
  padding-top: env(safe-area-inset-top, 0px);
  padding-left: env(safe-area-inset-left, 0px);
  padding-right: env(safe-area-inset-right, 0px);
}

.app-footer {
  padding-bottom: env(safe-area-inset-bottom, 0px);
}
```

When JavaScript needs the values, the API provides a snapshot:

```typescript
import { getSafeAreaInsets } from '@tauri-apps/api/safeArea'

const { top, right, bottom, left } = getSafeAreaInsets()
```

These values are in CSS pixels and describe the current document's viewport.
Do not multiply or divide them by `devicePixelRatio`. Re-read them after viewport
changes when using JavaScript calculations. A zero value can indicate an already
inset viewport or an engine that does not expose safe areas, not necessarily an
absent system bar. Native window avoid-area rectangles use a different coordinate
space and must not be applied directly to web content.

Existing generated projects need the `EntryAbility` layout change separately;
updating the JavaScript API alone does not change the native window layout.
