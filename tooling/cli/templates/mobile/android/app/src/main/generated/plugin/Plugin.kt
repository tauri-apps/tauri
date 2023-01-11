package {{reverse-domain app.domain}}.{{snake-case app.name}}

import android.webkit.WebView

abstract class Plugin {
  open fun load(webView: WebView) {}
}
