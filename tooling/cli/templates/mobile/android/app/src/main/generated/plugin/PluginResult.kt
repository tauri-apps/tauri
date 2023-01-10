package {{reverse-domain app.domain}}.{{snake-case app.name}}

import android.annotation.SuppressLint
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

  /**
   * Return plugin metadata and information about the result, if it succeeded the data, or error information if it didn't.
   * This is used for appRestoredResult, as it's technically a raw data response from a plugin.
   * @return the raw data response from the plugin.
   */
  val wrappedResult: JSObject
    get() {
      val ret = JSObject()
      ret.put("pluginId", json.getString("pluginId"))
      ret.put("methodName", json.getString("methodName"))
      ret.put("success", json.getBoolean("success", false))
      ret.put("data", json.getJSObject("data"))
      ret.put("error", json.getJSObject("error"))
      return ret
    }
}
