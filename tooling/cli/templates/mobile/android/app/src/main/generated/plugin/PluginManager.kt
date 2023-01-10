package {{reverse-domain app.domain}}.{{snake-case app.name}}

import android.annotation.SuppressLint
import android.net.Uri
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.webkit.JavaScriptReplyProxy
import androidx.webkit.WebMessageCompat
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewCompat.WebMessageListener
import androidx.webkit.WebViewFeature

class PluginManager(private val webView: WebView) {
  private val plugins: HashMap<String, PluginHandle> = HashMap()
  
  init {
    val plugin = SamplePlugin()
    plugins.put(plugin.interfaceName(), PluginHandle(plugin))
    load()
  }
  
  @SuppressLint("JavascriptInterface")
  private fun load() {
    if (WebViewFeature.isFeatureSupported(WebViewFeature.WEB_MESSAGE_LISTENER)
    ) {
      val messageListener =
        WebMessageListener { view: WebView?, message: WebMessageCompat, _: Uri?, isMainFrame: Boolean, _: JavaScriptReplyProxy ->
          if (isMainFrame) {
            postMessage(message.data ?: "")
          } else {
            Logger.warn("Plugin execution is allowed in Main Frame only")
          }
        }
      try {
        WebViewCompat.addWebMessageListener(
          webView,
          "__TAURI_PLUGIN_IPC__",
          mutableSetOf("https://tauri.localhost"),
          messageListener
        )
      } catch (ex: Exception) {
        webView.addJavascriptInterface(this, "__TAURI_PLUGIN_IPC__")
      }
    } else {
      webView.addJavascriptInterface(this, "__TAURI_PLUGIN_IPC__")
    }
    
    for (plugin in plugins) {
      webView.addJavascriptInterface(plugin.value.instance, plugin.value.instance.interfaceName())
      plugin.value.instance.load(webView)
    }
  }
  
  @JavascriptInterface
  fun postMessage(data: String) {
    val postData = JSObject(data)

    val callback: String = postData.getString("callback")
    val error: String = postData.getString("error")
    val pluginId: String = postData.getString("pluginId")
    val methodName: String = postData.getString("methodName")
    val methodData: JSObject? = postData.getJSObject("options", JSObject())

    Logger.verbose(
      Logger.tags("Plugin"),
      "Tauri plugin: pluginId: $pluginId, methodName: $methodName, callback: $callback, error: $error"
    )

    plugins[pluginId]?.invoke(methodName, PluginCall({
      call, successResult, errorResult -> 
        val (fn, result) = if (errorResult == null) Pair(callback, successResult) else Pair(error, errorResult)
        webView.evaluateJavascript("window['$fn']($result)", null)
    }, methodData))
  }
}