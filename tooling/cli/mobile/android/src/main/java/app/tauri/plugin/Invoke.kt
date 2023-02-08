package app.tauri.plugin

import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject
import app.tauri.Logger

class Invoke(
  private val sendResponse: (succcess: PluginResult?, error: PluginResult?) -> Unit,
  val data: JSObject?) {

  fun resolve(data: JSObject?) {
    val result = PluginResult(data)
    sendResponse(result, null)
  }

  fun resolve() {
    sendResponse(null, null)
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
    sendResponse(null, errorResult)
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
    return this.getString(name, null)
  }

  fun getString(name: String, defaultValue: String?): String? {
    val value = data!!.opt(name) ?: return defaultValue
    return if (value is String) {
      value
    } else defaultValue
  }

  fun getInt(name: String): Int? {
    return this.getInt(name, null)
  }

  fun getInt(name: String, defaultValue: Int?): Int? {
    val value = data!!.opt(name) ?: return defaultValue
    return if (value is Int) {
      value
    } else defaultValue
  }

  fun getLong(name: String): Long? {
    return this.getLong(name, null)
  }

  fun getLong(name: String, defaultValue: Long?): Long? {
    val value = data!!.opt(name) ?: return defaultValue
    return if (value is Long) {
      value
    } else defaultValue
  }

  fun getFloat(name: String): Float? {
    return this.getFloat(name, null)
  }

  fun getFloat(name: String, defaultValue: Float?): Float? {
    val value = data!!.opt(name) ?: return defaultValue
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
    return this.getDouble(name, null)
  }

  fun getDouble(name: String, defaultValue: Double?): Double? {
    val value = data!!.opt(name) ?: return defaultValue
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
    return this.getBoolean(name, null)
  }

  fun getBoolean(name: String, defaultValue: Boolean?): Boolean? {
    val value = data!!.opt(name) ?: return defaultValue
    return if (value is Boolean) {
      value
    } else defaultValue
  }

  fun getObject(name: String): JSObject? {
    return this.getObject(name, null)
  }

  fun getObject(name: String, defaultValue: JSObject?): JSObject? {
    val value = data!!.opt(name) ?: return defaultValue
    return if (value is JSObject) value else defaultValue
  }

  fun getArray(name: String): JSArray? {
    return this.getArray(name, null)
  }

  fun getArray(name: String, defaultValue: JSArray?): JSArray? {
    val value = data!!.opt(name) ?: return defaultValue
    return if (value is JSArray) value else defaultValue
  }

  fun hasOption(name: String): Boolean {
    return data!!.has(name)
  }
}
