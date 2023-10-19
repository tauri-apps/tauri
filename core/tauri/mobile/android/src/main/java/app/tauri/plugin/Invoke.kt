// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import app.tauri.Logger
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.module.SimpleModule
import com.fasterxml.jackson.module.kotlin.registerKotlinModule

class Invoke(
  val id: Long,
  val command: String,
  val callback: Long,
  val error: Long,
  private val sendResponse: (callback: Long, data: PluginResult?) -> Unit,
  private val sendChannelData: (channelId: Long, data: PluginResult) -> Unit,
  private val argsJson: String
) {

  fun<T> parseArgs(cls: Class<T>): T {
    val module = SimpleModule()
    module.addDeserializer(Channel::class.java, ChannelDeserializer(sendChannelData))
    return ObjectMapper().registerKotlinModule().registerModule(module).readValue(argsJson, cls)
  }

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
}
