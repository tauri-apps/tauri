package {{reverse-domain app.domain}}.{{snake-case app.name}}

import android.webkit.WebView

class PluginManager(private val webView: WebView) {
  private val plugins: HashMap<String, PluginHandle> = HashMap()
  
  fun load(name: String, plugin: Plugin) {
    plugin.load(webView)
    plugins[name] = PluginHandle(plugin)
  }
  
  fun postMessage(pluginId: String, methodName: String, data: JSObject, callback: String, error: String) {
    Logger.verbose(
      Logger.tags("Plugin"),
      "Tauri plugin: pluginId: $pluginId, methodName: $methodName, callback: $callback, error: $error"
    )

    plugins[pluginId]?.invoke(methodName, PluginCall({
      call, successResult, errorResult -> 
        val (fn, result) = if (errorResult == null) Pair(callback, successResult) else Pair(error, errorResult)
        webView.evaluateJavascript("window['$fn']($result)", null)
    }, data))
  }
}