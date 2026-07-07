---
"wry": minor
---

Add an expanded permission handling API for WebView2, WKWebView, WebKitGTK, and Android.
This includes:
- `PermissionKind` expansion: `DisplayCapture`, `Midi`, `Sensors`, `MediaKeySystemAccess`, `LocalFonts`, `WindowManagement`, `PointerLock`, `AutomaticDownloads`, `FileSystemAccess`, `Autoplay`.
- Support for `PermissionResponse::Prompt` to trigger native system dialogs.
- Android support via JNI bridge (`onPermissionRequest` in `RustWebChromeClient`).
- macOS: Split camera/microphone requests; `CameraAndMicrophone` resolved from individual responses.
- Linux: `DisplayCapture` detection for WebKitGTK < 2.42 (getDisplayMedia fix).
- Windows: Full coverage of all 12 `COREWEBVIEW2_PERMISSION_KIND` values.
