---
'tauri-runtime-cef': 'patch:enhance'
---

Disable Chrome browser features that do not belong in an application webview. Because the CEF runtime uses Chrome style browsers, every webview previously inherited a full Chrome profile and its behaviour: a "Save password?" bubble on any form submit, password leak detection sending a hashed prefix of typed credentials to Google, address and credit-card save bubbles, a "Translate this page?" bubble, and background requests to Google for alternate error pages and search suggestions. Each webview's request context now turns these off as soon as its profile finishes initializing. Safe Browsing is left enabled, because a Tauri webview routinely loads content the developer does not control — OAuth and SSO flows, embedded third-party pages, iframes and popups — and standard protection is a local hash-prefix database rather than a per-navigation lookup; a knob for applications that want to opt out is planned as a follow-up.
