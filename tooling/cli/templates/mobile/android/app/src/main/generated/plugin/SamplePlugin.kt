package {{reverse-domain app.domain}}.{{snake-case app.name}}

import android.webkit.JavascriptInterface
import android.webkit.WebView

class SamplePlugin: Plugin() {
  override fun interfaceName(): String {
    return "sample"
  }

  override fun load(webView: WebView) {
    println("loadddd!!!")
  }
  
  @JavascriptInterface
  @PluginMethod
  fun run(call: PluginCall) {
    println("running")
  }
}