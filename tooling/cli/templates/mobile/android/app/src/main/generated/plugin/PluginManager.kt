package {{reverse-domain app.domain}}.{{snake-case app.name}}

import android.net.Uri
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.webkit.JavaScriptReplyProxy
import androidx.webkit.WebMessageCompat
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewCompat.WebMessageListener
import androidx.webkit.WebViewFeature

class PluginManager {
  private val plugins: HashMap<String, PluginHandle> = HashMap()
  private var javaScriptReplyProxy: JavaScriptReplyProxy? = null
  
  init {
    val plugin = SamplePlugin()
    plugins.put(plugin.interfaceName(), PluginHandle(plugin))
  }
  
  fun load(webView: WebView) {
    if (WebViewFeature.isFeatureSupported(WebViewFeature.WEB_MESSAGE_LISTENER)
    ) {
      val messageListener =
        WebMessageListener { view: WebView?, message: WebMessageCompat, sourceOrigin: Uri?, isMainFrame: Boolean, replyProxy: JavaScriptReplyProxy ->
          if (isMainFrame) {
            postMessage(message.data ?: "")
            javaScriptReplyProxy = replyProxy
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
      webView.addJavascriptInterface(plugin, plugin.value.instance.interfaceName())
      plugin.value.instance.load(webView)
    }
  }
  
  @JavascriptInterface
  fun postMessage(data: String) {
    val postData = JSObject(data)

    val callbackId: String = postData.getString("callbackId")
    val pluginId: String = postData.getString("pluginId")
    val methodName: String = postData.getString("methodName")
    val methodData: JSObject? = postData.getJSObject("options", JSObject())

    Logger.verbose(
      Logger.tags("Plugin"),
      "Tauri plugin: callbackId: $callbackId, pluginId: $pluginId, methodName: $methodName"
    )

    plugins[pluginId]?.invoke(methodName, PluginCall())
  }
}