---
'tao': minor
---

`WindowEvent::ReceivedImeText` event's text is now coming from `ImmGetCompositionStringW` instead of a recording `WM_CHAR` and `WM_SYSCHAR` messages
