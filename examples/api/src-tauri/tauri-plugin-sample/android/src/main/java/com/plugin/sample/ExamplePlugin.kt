package com.plugin.sample

import android.app.Activity
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.PluginMethod
import app.tauri.plugin.TauriPlugin

@TauriPlugin
class ExamplePlugin(private val activity: Activity): Plugin() {
    private val implementation = Example()

    @PluginMethod
    fun ping(invoke: Invoke) {
        val value = invoke.getString("value") ?: ""
        val ret = JSObject()
        ret.put("value", implementation.pong(value))
        invoke.resolve(ret)
    }
}
