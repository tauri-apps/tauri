// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package com.plugin.sample

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

@InvokeArg
class PingArgs {
  var value: String? = null
  var onEvent: Channel? = null
}

class JsValues(val array: JSArray, val obj: JSObject)

@TauriPlugin
class ExamplePlugin(private val activity: Activity): Plugin(activity) {
    private val implementation = Example()

    @Command
    fun ping(invoke: Invoke) {
        val args = invoke.parseArgs(PingArgs::class.java)

        val event = JSObject()
        event.put("kind", "ping")
        args.onEvent?.send(event)

        val ret = JSObject()
        ret.put("value", implementation.pong(args.value ?: "default value :("))
        invoke.resolve(ret)
    }

    // Resolves org.json values through the plugin JSON mapper (`resolveObject`),
    // both as a property of a plain object and nested in one another.
    @Command
    fun jsValues(invoke: Invoke) {
        val array = JSArray()
        array.put("a")
        array.put(1)
        array.put(true)

        val obj = JSObject()
        obj.put("kind", "object")
        obj.put("nested", JSArray(listOf(1, 2)))

        invoke.resolveObject(JsValues(array, obj))
    }
}
