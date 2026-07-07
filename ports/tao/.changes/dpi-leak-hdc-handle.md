---
tao: patch
---

Fix getting the DPI internally leaks `HDC` handles on Windows. Also only call `GetDC` when on < Windows 8.1 which improves its performance.
