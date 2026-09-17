// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import android.content.res.Configuration
import android.content.Context
import android.content.Intent
import android.webkit.WebView
import androidx.activity.result.ActivityResult
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.IntentSenderRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import app.tauri.annotation.InvokeArg
import app.tauri.FsUtils
import app.tauri.JniMethod
import app.tauri.Logger
import com.fasterxml.jackson.annotation.JsonAutoDetect
import com.fasterxml.jackson.annotation.PropertyAccessor
import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.module.SimpleModule
import java.lang.reflect.InvocationTargetException

object PluginManager {
  fun interface RequestPermissionsCallback {
    fun onResult(permissions: Map<String, Boolean>)
  }

  fun interface ActivityResultCallback {
    fun onResult(result: ActivityResult)
  }

  /** The result launchers belonging to one activity. */
  private class ResultLaunchers(
    val startActivityForResult: ActivityResultLauncher<Intent>,
    val startIntentSenderForResult: ActivityResultLauncher<IntentSenderRequest>,
    val requestPermissions: ActivityResultLauncher<Array<String>>
  )

  // Insertion ordered, so the activity taken over when the current one goes away is the oldest
  // surviving one rather than an arbitrary member of a hash set.
  private val launchers: LinkedHashMap<AppCompatActivity, ResultLaunchers> = LinkedHashMap()
  var activity: AppCompatActivity? = null
  private val plugins: HashMap<String, PluginHandle> = HashMap()
  private var requestPermissionsCallback: RequestPermissionsCallback? = null
  private var startActivityForResultCallback: ActivityResultCallback? = null
  private var startIntentSenderForResultCallback: ActivityResultCallback? = null
  private var jsonMapper: ObjectMapper = ObjectMapper()
    .disable(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES)
    .enable(DeserializationFeature.FAIL_ON_NULL_FOR_PRIMITIVES)
    .setVisibility(PropertyAccessor.FIELD, JsonAutoDetect.Visibility.ANY)

  init {
    val channelDeserializer = ChannelDeserializer({ channelId, payload ->
      sendChannelData(channelId, payload)
    }, jsonMapper)
    jsonMapper
      .registerModule(SimpleModule().addDeserializer(Channel::class.java, channelDeserializer))
  }

  fun onCreate(activity: AppCompatActivity) {
    // Every activity gets its own launchers, and gets them here: registerForActivityResult must
    // be called before its owner reaches STARTED, so an activity that is already running can
    // never be given launchers later.
    launchers[activity] = registerResultLaunchers(activity)
    if (this.activity == null) {
      this.activity = activity
    }
  }

  private fun registerResultLaunchers(activity: AppCompatActivity): ResultLaunchers =
    ResultLaunchers(
      activity.registerForActivityResult(ActivityResultContracts.StartActivityForResult()
      ) { result ->
        startActivityForResultCallback?.onResult(result)
      },

      activity.registerForActivityResult(ActivityResultContracts.StartIntentSenderForResult()
      ) { result ->
        startIntentSenderForResultCallback?.onResult(result)
      },

      activity.registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions()
      ) { result ->
        requestPermissionsCallback?.onResult(result)
      }
    )

  private val currentLaunchers: ResultLaunchers
    get() = launchers[activity]
      ?: throw IllegalStateException("the plugin manager has no activity to launch from")

  fun onNewIntent(intent: Intent) {
    for (plugin in plugins.values) {
      plugin.instance.onNewIntent(intent)
    }
  }

  fun onPause(activity: AppCompatActivity) {
    for (plugin in plugins.values) {
      plugin.instance.triggerOnPause(activity)
    }
  }

  fun onResume(activity: AppCompatActivity) {
    for (plugin in plugins.values) {
      plugin.instance.triggerOnResume(activity)
    }
  }

  fun onRestart(activity: AppCompatActivity) {
    for (plugin in plugins.values) {
      plugin.instance.triggerOnRestart(activity)
    }
  }

  fun onStop(activity: AppCompatActivity) {
    for (plugin in plugins.values) {
      plugin.instance.triggerOnStop(activity)
    }
  }

  fun onDestroy(activity: AppCompatActivity) {
    for (plugin in plugins.values) {
      plugin.instance.triggerOnDestroy(activity)
    }

    launchers.remove(activity)
    if (this.activity == activity) {
      // Whatever is left already holds its own launchers, registered when it was created. Moving
      // this activity's launchers over instead would mean registering against an activity that is
      // already running, which registerForActivityResult rejects with an IllegalStateException.
      this.activity = launchers.keys.firstOrNull()
    }
  }

  fun onConfigurationChanged(newConfig: Configuration) {
    for (plugin in plugins.values) {
      plugin.instance.onConfigurationChanged(newConfig)
    }
  }

  fun startActivityForResult(intent: Intent, callback: ActivityResultCallback) {
    startActivityForResultCallback = callback
    currentLaunchers.startActivityForResult.launch(intent)
  }

  fun startIntentSenderForResult(intent: IntentSenderRequest, callback: ActivityResultCallback) {
    startIntentSenderForResultCallback = callback
    currentLaunchers.startIntentSenderForResult.launch(intent)
  }

  fun requestPermissions(
    permissionStrings: Array<String>,
    callback: RequestPermissionsCallback
  ) {
    requestPermissionsCallback = callback
    currentLaunchers.requestPermissions.launch(permissionStrings)
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

  fun<T> loadConfig(context: Context, plugin: String, cls: Class<T>): T {
    val tauriConfigJson = FsUtils.readAsset(context.assets, "tauri.conf.json")
    val mapper = ObjectMapper()
      .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
    val config = mapper.readValue(tauriConfigJson, Config::class.java)
    return mapper.readValue(config.plugins[plugin].toString(), cls)
  }

  private external fun handlePluginResponse(id: Int, success: String?, error: String?)
  private external fun sendChannelData(id: Long, data: String)
}

@InvokeArg
internal class Config {
  lateinit var plugins: Map<String, JsonNode>
}
