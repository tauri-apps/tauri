package {{reverse-domain app.domain}}.{{snake-case app.name}}

import android.webkit.WebView

abstract class Plugin {
  abstract fun interfaceName(): String
  
  open fun load(webView: WebView) {}
}
