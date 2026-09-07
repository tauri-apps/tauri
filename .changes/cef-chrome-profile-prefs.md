---
'tauri-runtime-cef': 'patch:enhance'
---

Disable Chrome browser features that do not belong in an application webview. Because the CEF runtime uses Chrome style browsers, every webview previously inherited a full Chrome profile and its behaviour: a "Save password?" bubble on any form submit, address and credit-card save bubbles, a "Translate this page?" bubble, and background requests to Google for alternate error pages, search suggestions, and Safe Browsing lookups. Each webview's request context now turns these off as soon as its profile finishes initializing. Safe Browsing is off because an app webview loads the application's own bundled content; an opt-in knob for apps that browse arbitrary remote web content is planned as a follow-up.
