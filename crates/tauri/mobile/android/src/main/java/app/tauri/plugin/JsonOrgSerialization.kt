// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import com.fasterxml.jackson.core.JsonGenerator
import com.fasterxml.jackson.core.JsonParser
import com.fasterxml.jackson.databind.DeserializationContext
import com.fasterxml.jackson.databind.JsonDeserializer
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.JsonSerializer
import com.fasterxml.jackson.databind.SerializerProvider

// Referenced from the class annotations on JSObject and JSArray, so these types
// (de)serialize with any ObjectMapper: a plugin's own, or one a broadcast receiver
// creates without the plugin loaded. JsonOrgModule only covers JSONObject/JSONArray
// themselves, so a `JSObject` field used to be filled through JSONObject's private
// fields and came out empty.

internal class JSObjectSerializer : JsonSerializer<JSObject>() {
  override fun serialize(value: JSObject, gen: JsonGenerator, serializers: SerializerProvider) {
    gen.writeRawValue(value.toString())
  }
}

internal class JSObjectDeserializer : JsonDeserializer<JSObject>() {
  override fun deserialize(parser: JsonParser, context: DeserializationContext): JSObject {
    val node = parser.codec.readTree<JsonNode>(parser)
    if (!node.isObject) {
      return context.reportInputMismatch(JSObject::class.java, "expected a JSON object, got %s", node.nodeType)
    }
    return JSObject(node.toString())
  }
}

internal class JSArraySerializer : JsonSerializer<JSArray>() {
  override fun serialize(value: JSArray, gen: JsonGenerator, serializers: SerializerProvider) {
    gen.writeRawValue(value.toString())
  }
}

internal class JSArrayDeserializer : JsonDeserializer<JSArray>() {
  override fun deserialize(parser: JsonParser, context: DeserializationContext): JSArray {
    val node = parser.codec.readTree<JsonNode>(parser)
    if (!node.isArray) {
      return context.reportInputMismatch(JSArray::class.java, "expected a JSON array, got %s", node.nodeType)
    }
    return JSArray(node.toString())
  }
}
