// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/* THIS FILE IS AUTO-GENERATED. DO NOT MODIFY!! */

package {{package}}

import android.os.Bundle
import android.content.Intent
import app.tauri.plugin.PluginManager

abstract class TauriActivity : WryActivity() {
  var pluginManager: PluginManager = PluginManager(this)

  override fun onNewIntent(intent: Intent) {
    super.onNewIntent(intent)
    pluginManager.onNewIntent(intent)
  }

  override fun onResume() {
    super.onResume()
    pluginManager.onResume()
  }

  override fun onPause() {
    super.onPause()
    pluginManager.onPause()
  }
}
