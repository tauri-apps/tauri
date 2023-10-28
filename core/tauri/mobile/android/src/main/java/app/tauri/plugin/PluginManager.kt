// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.webkit.WebView
import androidx.activity.result.ActivityResult
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import app.tauri.annotation.InvokeArg
import app.tauri.FsUtils
import app.tauri.JniMethod
import app.tauri.Logger
import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.module.SimpleModule
import java.lang.reflect.InvocationTargetException

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
  private var jsonMapper: ObjectMapper

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

    jsonMapper = ObjectMapper()
      .disable(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES)
      .enable(DeserializationFeature.FAIL_ON_NULL_FOR_PRIMITIVES)

    val channelDeserializer = ChannelDeserializer({ channelId, payload ->
      sendChannelData(channelId, payload)
    }, jsonMapper)
    jsonMapper
      .registerModule(SimpleModule().addDeserializer(Channel::class.java, channelDeserializer))
  }

  fun onNewIntent(intent: Intent) {
    for (plugin in plugins.values) {
      plugin.instance.onNewIntent(intent)
    }
  }

  fun onPause() {
    for (plugin in plugins.values) {
      plugin.instance.onPause()
    }
  }

  fun onResume() {
    for (plugin in plugins.values) {
      plugin.instance.onResume()
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
  fun load(webView: WebView?, name: String, plugin: Plugin, config: String) {
    val handle = PluginHandle(this, name, plugin, config, jsonMapper)
    plugins[name] = handle
    if (webView != null) {
      plugin.load(webView)
    }
  }

  @JniMethod
  fun runCommand(id: Int, pluginId: String, command: String, data: String) {
    val successId = 0L
    val errorId = 1L
    val invoke = Invoke(id.toLong(), command, successId, errorId, { fn, result ->
      var success: String? = null
      var error: String? = null
      if (fn == successId) {
        success = result
      } else {
        error = result
      }
      handlePluginResponse(id, success, error)
    }, data, jsonMapper)

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
      var exception: Throwable = e
      if (exception.message?.isEmpty() != false) {
        if (e is InvocationTargetException) {
          exception = e.targetException
        }
      }
      invoke.reject(if (exception.message?.isEmpty() != false) { exception.toString() } else { exception.message })
    }
  }

  companion object {
    fun<T> loadConfig(context: Context, plugin: String, cls: Class<T>): T {
      val tauriConfigJson = FsUtils.readAsset(context.assets, "tauri.conf.json")
      val mapper = ObjectMapper()
        .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
      val config = mapper.readValue(tauriConfigJson, Config::class.java)
      return mapper.readValue(config.plugins[plugin].toString(), cls)
    }
  }

  private external fun handlePluginResponse(id: Int, success: String?, error: String?)
  private external fun sendChannelData(id: Long, data: String)
}

@InvokeArg
internal class Config {
  lateinit var plugins: Map<String, JsonNode>
}
