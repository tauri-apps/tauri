// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/* THIS FILE IS AUTO-GENERATED. DO NOT MODIFY!! */

package {{package}}

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
