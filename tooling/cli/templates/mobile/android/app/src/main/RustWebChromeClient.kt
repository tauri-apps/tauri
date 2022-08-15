package {{reverse-domain app.domain}}.{{snake-case app.name}}

import android.webkit.*

class RustWebChromeClient: WebChromeClient() {
  private var loadedUrl: String? = null

  override fun onProgressChanged(view: WebView, progress: Int) {
    var url = view.url ?: ""
    if (url.endsWith("##")) {
      url = url.dropLast(2)
    }
    if (loadedUrl != url) {
      loadedUrl = url
      runInitializationScripts()
    }
  }

  companion object {
    init {
      System.loadLibrary("{{snake-case app.name}}")
    }
  }

  private external fun runInitializationScripts()
}
