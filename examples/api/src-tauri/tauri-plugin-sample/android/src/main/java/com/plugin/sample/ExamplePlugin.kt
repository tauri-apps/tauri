// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package com.plugin.sample

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

@TauriPlugin
class ExamplePlugin(private val activity: Activity): Plugin(activity) {
    private val implementation = Example()

    @Command
    fun ping(invoke: Invoke) {
        val onEvent = invoke.getChannel("onEvent")
        val event = JSObject()
        event.put("kind", "ping")
        onEvent?.send(event)

        val value = invoke.getString("value") ?: ""
        val ret = JSObject()
        ret.put("value", implementation.pong(value))
        invoke.resolve(ret)
    }
}
