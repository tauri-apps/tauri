package {{reverse-domain app.domain}}.{{snake-case app.name}}

import android.webkit.*

class Ipc {
    @JavascriptInterface
    fun postMessage(message: String) {
        this.ipc(message)
    }

    companion object {
        init {
            System.loadLibrary("{{snake-case app.name}}")
        }
    }

    private external fun ipc(message: String)
}
