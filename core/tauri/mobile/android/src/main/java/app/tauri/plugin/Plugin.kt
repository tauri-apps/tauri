package app.tauri.plugin

import android.webkit.WebView

abstract class Plugin {
  open fun load(webView: WebView) {}
}
