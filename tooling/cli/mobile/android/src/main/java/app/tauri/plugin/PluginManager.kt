package app.tauri.plugin

import android.webkit.WebView
import app.tauri.Logger

class PluginManager {
  private val plugins: HashMap<String, PluginHandle> = HashMap()

  fun onWebViewCreated(webView: WebView) {
    for ((_, plugin) in plugins) {
      if (!plugin.loaded) {
        plugin.load(webView)
      }
    }
  }

  fun load(webView: WebView?, name: String, plugin: Plugin) {
    val handle = PluginHandle(plugin)
    plugins[name] = handle
    if (webView != null) {
      plugin.load(webView)
    }
  }

  fun postMessage(webView: WebView, pluginId: String, methodName: String, data: JSObject, callback: Long, error: Long) {
    Logger.verbose(
      Logger.tags("Plugin"),
      "Tauri plugin: pluginId: $pluginId, methodName: $methodName, callback: $callback, error: $error"
    )

    val invoke = Invoke({ successResult, errorResult ->
      val (fn, result) = if (errorResult == null) Pair(callback, successResult) else Pair(
        error,
        errorResult
      )
      webView.evaluateJavascript("window['_$fn']($result)", null)
    }, data)
    try {
      val plugin = plugins[pluginId]
      if (plugin == null) {
        invoke.reject("Plugin $pluginId not initialized")
      } else {
        plugins[pluginId]?.invoke(methodName, invoke)
      }
    } catch (e: Exception) {
      invoke.reject(e.toString())
    }
  }
}
