---
'tao': patch
---

Fix iPadOS 26 system window controls overlapping `WKWebView` content by implementing `preferredWindowingControlStyleForScene:` on the scene delegate and returning the `minimal` style. The optional protocol method is a no-op on iOS versions earlier than 26.
