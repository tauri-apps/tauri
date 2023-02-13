package app.tauri.plugin

import android.webkit.WebView
import app.tauri.JniMethod
import app.tauri.Logger

class PluginManager {
  private val plugins: HashMap<String, PluginHandle> = HashMap()

  @JniMethod
  fun onWebViewCreated(webView: WebView) {
    for ((_, plugin) in plugins) {
      if (!plugin.loaded) {
        plugin.load(webView)
      }
    }
  }

  @JniMethod
  fun load(webView: WebView?, name: String, plugin: Plugin) {
    val handle = PluginHandle(plugin)
    plugins[name] = handle
    if (webView != null) {
      plugin.load(webView)
    }
  }

  @JniMethod
  fun postIpcMessage(webView: WebView, pluginId: String, methodName: String, data: JSObject, callback: Long, error: Long) {
    val invoke = Invoke({ successResult, errorResult ->
      val (fn, result) = if (errorResult == null) Pair(callback, successResult) else Pair(
        error,
        errorResult
      )
      webView.evaluateJavascript("window['_$fn']($result)", null)
    }, data)

    dispatchPluginMessage(invoke, pluginId, methodName)
  }

  @JniMethod
  fun runPluginMethod(id: Int, pluginId: String, methodName: String, data: JSObject) {
    val invoke = Invoke({ successResult, errorResult ->
      handlePluginResponse(id, successResult?.toString(), errorResult?.toString())
    }, data)

    dispatchPluginMessage(invoke, pluginId, methodName)
  }

  private fun dispatchPluginMessage(invoke: Invoke, pluginId: String, methodName: String) {
    Logger.verbose(
      Logger.tags("Plugin"),
      "Tauri plugin: pluginId: $pluginId, methodName: $methodName"
    )

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

  private external fun handlePluginResponse(id: Int, success: String?, error: String?)
}
