---
'tauri': 'minor:feat'
---

Add public API to access isolation pattern UUID for CSP configuration.

Added new methods to `AppHandle` to retrieve the isolation pattern UUID and formatted frame source URL:
- `AppHandle::isolation_uuid()` - Returns the raw isolation schema UUID
- `AppHandle::isolation_frame_src(use_https_scheme)` - Returns the formatted frame source URL for CSP configuration

Also added helper methods to the `Pattern` enum:
- `Pattern::isolation_schema()` - Returns the isolation schema if using isolation pattern
- `Pattern::isolation_frame_src(use_https_scheme)` - Returns formatted isolation frame source URL

This allows developers to properly configure Content-Security-Policy `frame-src` directives when dynamically creating WebViews with custom content, solving the issue where isolation iframes were blocked due to unknown frame sources.