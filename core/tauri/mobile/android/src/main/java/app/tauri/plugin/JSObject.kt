// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import org.json.JSONException
import org.json.JSONObject

class JSObject : JSONObject {
  constructor() : super()
  constructor(json: String) : super(json)
  constructor(obj: JSONObject, names: Array<String>) : super(obj, names)

  override fun getString(key: String): String {
    return getString(key, "")
  }

  fun getString(key: String, defaultValue: String): String {
    try {
      val value = super.getString(key)
      if (!super.isNull(key)) {
        return value
      }
    } catch (_: JSONException) {
    }
    return defaultValue
  }

  fun getInteger(key: String): Int? {
    return getInteger(key, null)
  }

  fun getInteger(key: String, defaultValue: Int?): Int? {
    try {
      return super.getInt(key)
    } catch (_: JSONException) {
    }
    return defaultValue
  }

  fun getBoolean(key: String, defaultValue: Boolean?): Boolean? {
    try {
      return super.getBoolean(key)
    } catch (_: JSONException) {
    }
    return defaultValue
  }

  /**
   * Fetch boolean from jsonObject
   */
  fun getBool(key: String): Boolean? {
    return getBoolean(key, null)
  }

  fun getJSObject(name: String): JSObject? {
    try {
      return getJSObject(name, null)
    } catch (e: JSONException) {
    }
    return null
  }

  @Throws(JSONException::class)
  fun getJSObject(name: String, defaultValue: JSObject?): JSObject? {
    try {
      val obj = get(name)
      if (obj is JSONObject) {
        val keysIter = obj.keys()
        val keys: MutableList<String> = ArrayList()
        while (keysIter.hasNext()) {
          keys.add(keysIter.next())
        }
        return JSObject(obj, keys.toTypedArray())
      }
    } catch (_: JSONException) {
    }
    return defaultValue
  }

  override fun put(key: String, value: Boolean): JSObject {
    try {
      super.put(key, value)
    } catch (_: JSONException) {
    }
    return this
  }

  override fun put(key: String, value: Int): JSObject {
    try {
      super.put(key, value)
    } catch (_: JSONException) {
    }
    return this
  }

  override fun put(key: String, value: Long): JSObject {
    try {
      super.put(key, value)
    } catch (_: JSONException) {
    }
    return this
  }

  override fun put(key: String, value: Double): JSObject {
    try {
      super.put(key, value)
    } catch (_: JSONException) {
    }
    return this
  }

  override fun put(key: String, value: Any?): JSObject {
    try {
      super.put(key, value)
    } catch (_: JSONException) {
    }
    return this
  }

  fun put(key: String, value: String?): JSObject {
    try {
      super.put(key, value)
    } catch (_: JSONException) {
    }
    return this
  }

  @Throws(JSONException::class)
  fun putSafe(key: String, value: Any?): JSObject {
    return super.put(key, value) as JSObject
  }

  companion object {
    /**
     * Convert a pathetic JSONObject into a JSObject
     * @param obj
     */
    @Throws(JSONException::class)
    fun fromJSONObject(obj: JSONObject): JSObject {
      val keysIter = obj.keys()
      val keys: MutableList<String> = ArrayList()
      while (keysIter.hasNext()) {
        keys.add(keysIter.next())
      }
      return JSObject(obj, keys.toTypedArray())
    }
  }
}
