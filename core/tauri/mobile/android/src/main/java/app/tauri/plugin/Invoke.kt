// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import app.tauri.Logger

const val CHANNEL_PREFIX = "__CHANNEL__:"

class Invoke(
  val id: Long,
  val command: String,
  val callback: Long,
  val error: Long,
  private val sendResponse: (callback: Long, data: PluginResult?) -> Unit,
  private val sendChannelData: (channelId: Long, data: PluginResult) -> Unit,
  val data: JSObject) {

  fun resolve(data: JSObject?) {
    val result = PluginResult(data)
    sendResponse(callback, result)
  }

  fun resolve() {
    sendResponse(callback, null)
  }

  fun reject(msg: String?, code: String?, ex: Exception?, data: JSObject?) {
    val errorResult = PluginResult()
    if (ex != null) {
      Logger.error(Logger.tags("Plugin"), msg!!, ex)
    }
    try {
      errorResult.put("message", msg)
      errorResult.put("code", code)
      if (null != data) {
        errorResult.put("data", data)
      }
    } catch (jsonEx: Exception) {
      Logger.error(Logger.tags("Plugin"), jsonEx.message!!, jsonEx)
    }
    sendResponse(error, errorResult)
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

  fun getString(name: String): String? {
    return getStringInternal(name, null)
  }

  fun getString(name: String, defaultValue: String): String {
    return getStringInternal(name, defaultValue)!!
  }

  private fun getStringInternal(name: String, defaultValue: String?): String? {
    val value = data.opt(name) ?: return defaultValue
    return if (value is String) {
      value
    } else defaultValue
  }

  fun getInt(name: String): Int? {
    return getIntInternal(name, null)
  }

  fun getInt(name: String, defaultValue: Int): Int {
    return getIntInternal(name, defaultValue)!!
  }

  private fun getIntInternal(name: String, defaultValue: Int?): Int? {
    val value = data.opt(name) ?: return defaultValue
    return if (value is Int) {
      value
    } else defaultValue
  }

  fun getLong(name: String): Long? {
    return getLongInternal(name, null)
  }

  fun getLong(name: String, defaultValue: Long): Long {
    return getLongInternal(name, defaultValue)!!
  }

  private fun getLongInternal(name: String, defaultValue: Long?): Long? {
    val value = data.opt(name) ?: return defaultValue
    return if (value is Long) {
      value
    } else defaultValue
  }

  fun getFloat(name: String): Float? {
    return getFloatInternal(name, null)
  }

  fun getFloat(name: String, defaultValue: Float): Float {
    return getFloatInternal(name, defaultValue)!!
  }

  private fun getFloatInternal(name: String, defaultValue: Float?): Float? {
    val value = data.opt(name) ?: return defaultValue
    if (value is Float) {
      return value
    }
    if (value is Double) {
      return value.toFloat()
    }
    return if (value is Int) {
      value.toFloat()
    } else defaultValue
  }

  fun getDouble(name: String): Double? {
    return getDoubleInternal(name, null)
  }

  fun getDouble(name: String, defaultValue: Double): Double {
    return getDoubleInternal(name, defaultValue)!!
  }

  private fun getDoubleInternal(name: String, defaultValue: Double?): Double? {
    val value = data.opt(name) ?: return defaultValue
    if (value is Double) {
      return value
    }
    if (value is Float) {
      return value.toDouble()
    }
    return if (value is Int) {
      value.toDouble()
    } else defaultValue
  }

  fun getBoolean(name: String): Boolean? {
    return getBooleanInternal(name, null)
  }

  fun getBoolean(name: String, defaultValue: Boolean): Boolean {
    return getBooleanInternal(name, defaultValue)!!
  }

  private fun getBooleanInternal(name: String, defaultValue: Boolean?): Boolean? {
    val value = data.opt(name) ?: return defaultValue
    return if (value is Boolean) {
      value
    } else defaultValue
  }

  fun getObject(name: String): JSObject? {
    return getObjectInternal(name, null)
  }

  fun getObject(name: String, defaultValue: JSObject): JSObject {
    return getObjectInternal(name, defaultValue)!!
  }

  private fun getObjectInternal(name: String, defaultValue: JSObject?): JSObject? {
    val value = data.opt(name) ?: return defaultValue
    return if (value is JSObject) value else defaultValue
  }

  fun getArray(name: String): JSArray? {
    return getArrayInternal(name, null)
  }

  fun getArray(name: String, defaultValue: JSArray): JSArray {
    return getArrayInternal(name, defaultValue)!!
  }

  private fun getArrayInternal(name: String, defaultValue: JSArray?): JSArray? {
    val value = data.opt(name) ?: return defaultValue
    return if (value is JSArray) value else defaultValue
  }

  fun hasOption(name: String): Boolean {
    return data.has(name)
  }

  fun getChannel(name: String): Channel? {
    val channelDef = getString(name, "")
    val callback = channelDef.substring(CHANNEL_PREFIX.length).toLongOrNull() ?: return null
    return Channel(callback) { res -> sendChannelData(callback, PluginResult(res)) }
  }
}
