package {{reverse-domain app.domain}}.{{snake-case app.name}}

import app.tauri.plugin.PluginManager

abstract class TauriActivity : WryActivity() {
  var pluginManager: PluginManager = PluginManager(this)
}
