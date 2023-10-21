// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import app.tauri.Logger
import com.fasterxml.jackson.core.type.TypeReference
import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.module.SimpleModule
import com.fasterxml.jackson.module.kotlin.registerKotlinModule

// Just a marker to differentiate on resolve()
interface InvokeResponse {}

class Invoke(
  val id: Long,
  val command: String,
  val callback: Long,
  val error: Long,
  private val sendResponse: (callback: Long, data: String) -> Unit,
  private val sendChannelData: (channelId: Long, data: String) -> Unit,
  private val argsJson: String
) {

  private fun objectMapper(): ObjectMapper {
    return ObjectMapper()
      .registerKotlinModule()
      .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
      .registerModule(SimpleModule().addDeserializer(Channel::class.java, ChannelDeserializer(sendChannelData)))
  }

  fun<T> parseArgs(cls: Class<T>): T {
    return objectMapper().readValue(argsJson, cls)
  }

  fun<T> parseArgs(ref: TypeReference<T>): T {
    return objectMapper().readValue(argsJson, ref)
  }

  fun resolve(data: JSObject?) {
    sendResponse(callback, PluginResult(data).toString())
  }

  fun<T: InvokeResponse> resolve(data: T) {
    sendResponse(
      callback,
      ObjectMapper()
      .registerKotlinModule()
      .writeValueAsString(data)
    )
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
