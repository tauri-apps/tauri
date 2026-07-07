---
"wry": patch
---

On macOS, avoid an extra copy for owned custom protocol response bodies by transferring the body buffer into `NSData`.
