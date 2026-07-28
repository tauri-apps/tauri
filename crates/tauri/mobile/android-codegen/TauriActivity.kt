// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/* THIS FILE IS AUTO-GENERATED. DO NOT MODIFY!! */

package {{package}}

import android.content.Intent
import android.content.res.Configuration
import android.os.Bundle
import app.tauri.plugin.PluginManager

abstract class TauriActivity : WryActivity() {
  override val handleBackNavigation: Boolean = false

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    PluginManager.onActivityCreate(this)
  }

  fun getPluginManager(): PluginManager {
    return PluginManager
  }

  override fun onNewIntent(intent: Intent) {
    super.onNewIntent(intent)
    PluginManager.onNewIntent(intent)
  }

  override fun onRestart() {
    super.onRestart()
    PluginManager.onRestart(this)
  }

  override fun onResume() {
    super.onResume()
    PluginManager.onResume(this)
  }

  override fun onPause() {
    super.onPause()
    PluginManager.onPause(this)
  }

  override fun onStop() {
    super.onStop()
    PluginManager.onStop(this)
  }

  override fun onDestroy() {
    super.onDestroy()
    PluginManager.onDestroy(this)
  }

  override fun onConfigurationChanged(newConfig: Configuration) {
    super.onConfigurationChanged(newConfig)
    PluginManager.onConfigurationChanged(newConfig)
  }
}
