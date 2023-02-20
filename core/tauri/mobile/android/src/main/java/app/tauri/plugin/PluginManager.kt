// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import android.content.Intent
import android.webkit.WebView
import androidx.activity.result.ActivityResult
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import app.tauri.JniMethod
import app.tauri.Logger

class PluginManager(val activity: AppCompatActivity) {
  fun interface RequestPermissionsCallback {
    fun onResult(permissions: Map<String, Boolean>)
  }

  fun interface ActivityResultCallback {
    fun onResult(result: ActivityResult)
  }

  private val plugins: HashMap<String, PluginHandle> = HashMap()
  private val startActivityForResultLauncher: ActivityResultLauncher<Intent>
  private val requestPermissionsLauncher: ActivityResultLauncher<Array<String>>
  private var requestPermissionsCallback: RequestPermissionsCallback? = null
  private var startActivityForResultCallback: ActivityResultCallback? = null

  init {
    startActivityForResultLauncher =
      activity.registerForActivityResult(ActivityResultContracts.StartActivityForResult()
      ) { result ->
        if (startActivityForResultCallback != null) {
          startActivityForResultCallback!!.onResult(result)
        }
      }

    requestPermissionsLauncher =
      activity.registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions()
      ) { result ->
        if (requestPermissionsCallback != null) {
          requestPermissionsCallback!!.onResult(result)
        }
      }
  }

  fun startActivityForResult(intent: Intent, callback: ActivityResultCallback) {
    startActivityForResultCallback = callback
    startActivityForResultLauncher.launch(intent)
  }

  fun requestPermissions(
    permissionStrings: Array<String>,
    callback: RequestPermissionsCallback
  ) {
    requestPermissionsCallback = callback
    requestPermissionsLauncher.launch(permissionStrings)
  }

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
    val handle = PluginHandle(this, name, plugin)
    plugins[name] = handle
    if (webView != null) {
      plugin.load(webView)
    }
  }

  @JniMethod
  fun postIpcMessage(webView: WebView, pluginId: String, command: String, data: JSObject, callback: Long, error: Long) {
    val invoke = Invoke(callback, command, { successResult, errorResult ->
      val (fn, result) = if (errorResult == null) Pair(callback, successResult) else Pair(
        error,
        errorResult
      )
      webView.evaluateJavascript("window['_$fn']($result)", null)
    }, data)

    dispatchPluginMessage(invoke, pluginId)
  }

  @JniMethod
  fun runPluginMethod(id: Int, pluginId: String, command: String, data: JSObject) {
    val invoke = Invoke(id.toLong(), command, { successResult, errorResult ->
      handlePluginResponse(id, successResult?.toString(), errorResult?.toString())
    }, data)

    dispatchPluginMessage(invoke, pluginId)
  }

  private fun dispatchPluginMessage(invoke: Invoke, pluginId: String) {
    Logger.verbose(
      Logger.tags("Plugin"),
      "Tauri plugin: pluginId: $pluginId, command: ${invoke.command}"
    )

    try {
      val plugin = plugins[pluginId]
      if (plugin == null) {
        invoke.reject("Plugin $pluginId not initialized")
      } else {
        plugins[pluginId]?.invoke(invoke)
      }
    } catch (e: Exception) {
      invoke.reject(e.toString())
    }
  }

  private external fun handlePluginResponse(id: Int, success: String?, error: String?)
}
