// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import android.annotation.SuppressLint
import app.tauri.Logger
import java.text.DateFormat
import java.text.SimpleDateFormat
import java.util.*

class PluginResult @JvmOverloads constructor(json: JSObject? = JSObject()) {
  private val json: JSObject

  init {
    this.json = json ?: JSObject()
  }

  fun put(name: String, value: Boolean): PluginResult {
    return jsonPut(name, value)
  }

  fun put(name: String, value: Double): PluginResult {
    return jsonPut(name, value)
  }

  fun put(name: String, value: Int): PluginResult {
    return jsonPut(name, value)
  }

  fun put(name: String, value: Long): PluginResult {
    return jsonPut(name, value)
  }

  /**
   * Format a date as an ISO string
   */
  @SuppressLint("SimpleDateFormat")
  fun put(name: String, value: Date): PluginResult {
    val tz: TimeZone = TimeZone.getTimeZone("UTC")
    val df: DateFormat = SimpleDateFormat("yyyy-MM-dd'T'HH:mm'Z'")
    df.timeZone = tz
    return jsonPut(name, df.format(value))
  }

  fun put(name: String, value: Any?): PluginResult {
    return jsonPut(name, value)
  }

  fun put(name: String, value: PluginResult): PluginResult {
    return jsonPut(name, value.json)
  }

  private fun jsonPut(name: String, value: Any?): PluginResult {
    try {
      json.put(name, value)
    } catch (ex: Exception) {
      Logger.error(Logger.tags("Plugin"), "", ex)
    }
    return this
  }

  override fun toString(): String {
    return json.toString()
  }
}
