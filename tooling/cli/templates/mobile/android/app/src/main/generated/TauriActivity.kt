package {{reverse-domain app.domain}}.{{snake-case app.name}}

import android.os.Bundle
import app.tauri.plugin.PluginManager

abstract class TauriActivity : WryActivity() {
  var pluginManager: PluginManager = PluginManager(this)

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    if (intent != null) {
      pluginManager.onNewIntent(intent)
    }
  }
}
