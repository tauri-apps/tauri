package {{reverse-domain app.domain}}.{{snake-case app.name}}

import android.graphics.Bitmap
import android.webkit.*

class RustWebViewClient(initScripts: Array<String>): WebViewClient() {
    private val initializationScripts: Array<String>
  
    init {
      initializationScripts = initScripts
    }

    override fun onPageStarted(view: WebView?, url: String?, favicon: Bitmap?) {
        for (script in initializationScripts) {
          view?.evaluateJavascript(script, null)
        }
        super.onPageStarted(view, url, favicon)
    }
  
    override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean {
        return false
    }

    override fun shouldInterceptRequest(
        view: WebView,
        request: WebResourceRequest
    ): WebResourceResponse? {
        return handleRequest(request)
    }

    companion object {
        init {
            System.loadLibrary("{{snake-case app.name}}")
        }
    }

    private external fun handleRequest(request: WebResourceRequest): WebResourceResponse?
}
