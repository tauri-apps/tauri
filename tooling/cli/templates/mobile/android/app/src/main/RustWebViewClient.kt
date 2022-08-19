package {{reverse-domain app.domain}}.{{snake-case app.name}}

import android.webkit.*

class RustWebViewClient: WebViewClient() {
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
