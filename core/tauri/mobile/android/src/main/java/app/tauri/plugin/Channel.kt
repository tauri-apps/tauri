// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import com.fasterxml.jackson.core.JsonParser
import com.fasterxml.jackson.databind.DeserializationContext
import com.fasterxml.jackson.databind.JsonDeserializer
import com.fasterxml.jackson.databind.ObjectMapper

const val CHANNEL_PREFIX = "__CHANNEL__:"

internal class ChannelDeserializer(val sendChannelData: (channelId: Long, data: String) -> Unit, private val objectMapper: ObjectMapper): JsonDeserializer<Channel>() {
  override fun deserialize(
    jsonParser: JsonParser?,
    deserializationContext: DeserializationContext
  ): Channel {
    val channelDef = deserializationContext.readValue(jsonParser, String::class.java)
    val callback = channelDef.substring(CHANNEL_PREFIX.length).toLongOrNull() ?: throw Error("unexpected channel value $channelDef")
    return Channel(callback, { res -> sendChannelData(callback, res) }, objectMapper)
  }
}

class Channel(val id: Long, private val handler: (data: String) -> Unit, private val objectMapper: ObjectMapper) {
  fun send(data: JSObject) {
    handler(PluginResult(data).toString())
  }

  fun sendObject(data: Any) {
    handler(objectMapper.writeValueAsString(data))
  }
}
