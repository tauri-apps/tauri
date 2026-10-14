// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import android.app.Activity
import app.tauri.Logger
import com.fasterxml.jackson.core.type.TypeReference
import com.fasterxml.jackson.databind.ObjectMapper
import java.lang.ref.WeakReference

class Invoke private constructor(
  val id: Long,
  val command: String,
  val callback: Long,
  val error: Long,
  private val sendResponse: (callback: Long, data: String) -> Unit,
  private val argsJson: String,
  private val jsonMapper: ObjectMapper,
  activity: Activity?,
  /**
   * Whether the call was made on behalf of a specific webview.
   * A contextual call must not fall back to another activity when [activity] is unusable.
   */
  val isContextual: Boolean
) {
  private val activityRef: WeakReference<Activity>? = activity?.let { WeakReference(it) }

  /**
   * The activity hosting the webview that originated this call.
   * Null for non-contextual calls or after the activity is garbage collected.
   */
  val activity: Activity?
    get() = activityRef?.get()

  constructor(
    id: Long,
    command: String,
    callback: Long,
    error: Long,
    sendResponse: (callback: Long, data: String) -> Unit,
    argsJson: String,
    jsonMapper: ObjectMapper
  ) : this(id, command, callback, error, sendResponse, argsJson, jsonMapper, null, false)

  internal constructor(
    id: Long,
    command: String,
    callback: Long,
    error: Long,
    sendResponse: (callback: Long, data: String) -> Unit,
    argsJson: String,
    jsonMapper: ObjectMapper,
    activity: Activity
  ) : this(id, command, callback, error, sendResponse, argsJson, jsonMapper, activity, true)

  fun getRawArgs(): String {
    return argsJson
  }

  fun getArgs(): JSObject {
    return JSObject(argsJson)
  }

  fun<T> parseArgs(cls: Class<T>): T {
    return jsonMapper.readValue(argsJson, cls)
  }

  fun<T> parseArgs(ref: TypeReference<T>): T {
    return jsonMapper.readValue(argsJson, ref)
  }

  fun resolve(data: JSObject?) {
    sendResponse(callback, PluginResult(data).toString())
  }

  fun resolveObject(data: Any) {
    sendResponse(callback, jsonMapper.writeValueAsString(data))
  }

  fun resolve() {
    sendResponse(callback, "null")
  }

  fun reject(msg: String?, code: String?, ex: Exception?, data: JSObject?) {
    val errorResult = PluginResult()

    if (ex != null) {
      Logger.error(Logger.tags("Plugin"), msg!!, ex)
    }

    errorResult.put("message", msg)
    if (code != null) {
      errorResult.put("code", code)
    }
    if (data != null) {
      errorResult.put("data", data)
    }

    sendResponse(error, errorResult.toString())
  }

  fun reject(msg: String?, ex: Exception?, data: JSObject?) {
    reject(msg, null, ex, data)
  }

  fun reject(msg: String?, code: String?, data: JSObject?) {
    reject(msg, code, null, data)
  }

  fun reject(msg: String?, code: String?, ex: Exception?) {
    reject(msg, code, ex, null)
  }

  fun reject(msg: String?, data: JSObject?) {
    reject(msg, null, null, data)
  }

  fun reject(msg: String?, ex: Exception?) {
    reject(msg, null, ex, null)
  }

  fun reject(msg: String?, code: String?) {
    reject(msg, code, null, null)
  }

  fun reject(msg: String?) {
    reject(msg, null, null, null)
  }
}
