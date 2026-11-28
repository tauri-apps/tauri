// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import com.fasterxml.jackson.databind.annotation.JsonDeserialize
import com.fasterxml.jackson.databind.annotation.JsonSerialize
import org.json.JSONArray
import org.json.JSONException

@JsonSerialize(using = JSArraySerializer::class)
@JsonDeserialize(using = JSArrayDeserializer::class)
class JSArray : JSONArray {
  constructor() : super() {}
  constructor(json: String?) : super(json) {}
  constructor(copyFrom: Collection<*>?) : super(copyFrom) {}
  constructor(array: Any?) : super(array) {}

  @Suppress("UNCHECKED_CAST", "ASSIGNED_BUT_NEVER_ACCESSED_VARIABLE")
  @Throws(JSONException::class)
  fun <E> toList(): List<E> {
    val items: MutableList<E> = ArrayList()
    var o: Any? = null
    for (i in 0 until this.length()) {
      this.get(i).also { o = it }
      try {
        items.add(this.get(i) as E)
      } catch (ex: Exception) {
        throw JSONException("Not all items are instances of the given type")
      }
    }
    return items
  }

  companion object {
    /**
     * Create a new JSArray without throwing a error
     */
    fun from(array: Any?): JSArray? {
      try {
        return JSArray(array)
      } catch (ex: JSONException) {
        //
      }
      return null
    }
  }
}
