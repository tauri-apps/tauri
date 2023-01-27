package com.plugin.test

import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.PluginMethod

class ExamplePlugin: Plugin() {
    private val implementation = Example()

    @PluginMethod
    fun echo(invoke: Invoke) {
        val value = invoke.getString("value") ?: ""
        val ret = JSObject()
        ret.put("value", implementation.echo(value))
        invoke.resolve(ret)
    }
}
