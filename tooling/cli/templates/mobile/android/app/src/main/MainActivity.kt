package {{reverse-domain app.domain}}.{{snake-case app.name}}

import app.tauri.plugin.PluginManager

class MainActivity : TauriActivity() {
  var pluginManager: PluginManager = PluginManager()
}
