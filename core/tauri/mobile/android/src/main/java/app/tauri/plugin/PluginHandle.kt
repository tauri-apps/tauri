// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import android.app.Activity
import android.content.Intent
import android.content.SharedPreferences
import android.webkit.WebView
import androidx.core.app.ActivityCompat
import app.tauri.PermissionHelper
import app.tauri.PermissionState
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.PluginMethod
import app.tauri.annotation.TauriPlugin
import java.lang.reflect.Method

class PluginHandle(private val manager: PluginManager, val name: String, private val instance: Plugin) {
  private val pluginMethods: HashMap<String, PluginMethodData> = HashMap()
  private val permissionCallbackMethods: HashMap<String, Method> = HashMap()
  private val startActivityCallbackMethods: HashMap<String, Method> = HashMap()
  var annotation: TauriPlugin?
  var loaded = false

  init {
    indexMethods()
    instance.handle = this
    annotation = instance.javaClass.getAnnotation(TauriPlugin::class.java)
  }

  fun load(webView: WebView) {
    instance.load(webView)
    loaded = true
  }

  fun startActivityForResult(invoke: Invoke, intent: Intent, callbackName: String) {
    manager.startActivityForResult(intent) { result ->
      val method = startActivityCallbackMethods[callbackName]
      if (method != null) {
        method.isAccessible = true
        method(instance, invoke, result)
      }
    }
  }

  fun requestPermissions(
    invoke: Invoke,
    permissions: Array<String>,
    callbackName: String
  ) {
    manager.requestPermissions(permissions) { result ->
      if (validatePermissions(invoke, result)) {
        val method = permissionCallbackMethods[callbackName]
        if (method != null) {
          method.isAccessible = true
          method(instance, invoke)
        }
      }
    }
  }

  /**
   * Saves permission states and rejects if permissions were not correctly defined in
   * the AndroidManifest.xml file.
   *
   * @param permissions
   * @return true if permissions were saved and defined correctly, false if not
   */
  private fun validatePermissions(
    invoke: Invoke,
    permissions: Map<String, Boolean>
  ): Boolean {
    val activity = manager.activity
    val prefs =
      activity.getSharedPreferences("PluginPermStates", Activity.MODE_PRIVATE)
    for ((permString, isGranted) in permissions) {
      if (isGranted) {
        // Permission granted. If previously denied, remove cached state
        val state = prefs.getString(permString, null)
        if (state != null) {
          val editor: SharedPreferences.Editor = prefs.edit()
          editor.remove(permString)
          editor.apply()
        }
      } else {
        val editor: SharedPreferences.Editor = prefs.edit()
        if (ActivityCompat.shouldShowRequestPermissionRationale(
            activity,
            permString
          )
        ) {
          // Permission denied, can prompt again with rationale
          editor.putString(permString, PermissionState.PROMPT_WITH_RATIONALE.toString())
        } else {
          // Permission denied permanently, store this state for future reference
          editor.putString(permString, PermissionState.DENIED.toString())
        }
        editor.apply()
      }
    }
    val permStrings = permissions.keys.toTypedArray()
    if (!PermissionHelper.hasDefinedPermissions(activity, permStrings)) {
      val builder = StringBuilder()
      builder.append("Missing the following permissions in AndroidManifest.xml:\n")
      val missing = PermissionHelper.getUndefinedPermissions(activity, permStrings)
      for (perm in missing) {
        builder.append(
          """
                $perm

                """.trimIndent()
        )
      }
      invoke.reject(builder.toString())
      return false
    }
    return true
  }

  @Throws(
    InvalidPluginMethodException::class,
    IllegalAccessException::class
  )
  fun invoke(invoke: Invoke) {
    val methodMeta = pluginMethods[invoke.command]
      ?: throw InvalidPluginMethodException("No command " + invoke.command + " found for plugin " + instance.javaClass.name)
    methodMeta.method.invoke(instance, invoke)
  }

  private fun indexMethods() {
    val methods: Array<Method> = instance.javaClass.methods
    for (method in methods) {
      if (method.isAnnotationPresent(PluginMethod::class.java)) {
        val pluginMethod = method.getAnnotation(PluginMethod::class.java) ?: continue
        val methodMeta = PluginMethodData(method, pluginMethod)
        pluginMethods[method.name] = methodMeta
      } else if (method.isAnnotationPresent(ActivityCallback::class.java)) {
        startActivityCallbackMethods[method.name] = method
      } else if (method.isAnnotationPresent(PermissionCallback::class.java)) {
        permissionCallbackMethods[method.name] = method
      }
    }
  }
}
