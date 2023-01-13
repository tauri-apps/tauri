package {{package_id}}

import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.PluginCall
import app.tauri.plugin.PluginMethod

class ExamplePlugin: Plugin() {
    private val implementation = Example()

    @PluginMethod
    fun echo(call: PluginCall) {
        val value = call.getString("value") ?: ""
        val ret = JSObject()
        ret.put("value", implementation.echo(value))
        call.resolve(ret)
    }
}
