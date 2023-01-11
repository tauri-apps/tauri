package {{reverse-domain app.domain}}.{{snake-case app.name}}

import android.webkit.WebView

class SamplePlugin: Plugin() {
  override fun load(webView: WebView) {
    println("loadddd!!!")
  }
  
  @PluginMethod
  fun run(call: PluginCall) {
    println("running")
    call.resolve()
  }
}