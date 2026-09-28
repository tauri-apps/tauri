---
"tauri": minor:feat
---

Added `PluginHandle::run_mobile_plugin_with_webview` and `PluginHandle::run_mobile_plugin_async_with_webview` to run a mobile plugin command on behalf of a webview. The native plugin gets the `UIViewController` (iOS) or `Activity` (Android) hosting that webview through `invoke.viewController` / `invoke.activity`, and `invoke.isContextual` is set, so plugins can present UI using the calling webview's controller or activity. If the webview is gone before the call reaches the native plugin, the call is rejected with the `ORIGIN_UNAVAILABLE` error code.

Plugins get the view controller or activity to present from with `Invoke.presentingViewController()` (iOS) and `Plugin.presentingActivity(invoke)` (Android): contextual calls get the calling one, or `nil`/`null` when the origin is unavailable. Without a webview context, these helpers preserve the existing behavior: iOS uses the controller of the most recently registered webview, and Android uses the activity the plugin was created with. The existing `run_mobile_plugin` APIs and plugin callbacks keep their current behavior.
