// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/* THIS FILE IS AUTO-GENERATED. DO NOT MODIFY!! */

package {{package}}

import android.content.Intent
import android.content.res.Configuration
import android.os.Bundle
import app.tauri.plugin.PluginManager
import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.ProcessLifecycleOwner

object TauriLifecycleObserver : DefaultLifecycleObserver {
    override fun onResume(owner: LifecycleOwner) {
      super.onResume(owner)
      PluginManager.onResume()
    }

    override fun onPause(owner: LifecycleOwner) {
      super.onPause(owner)
      PluginManager.onPause()
    }

    override fun onStop(owner: LifecycleOwner) {
      super.onStop(owner)
      PluginManager.onStop()
    }
}

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

  override fun onDestroy() {
    super.onDestroy()
    PluginManager.onDestroy(this)
  }

  override fun onConfigurationChanged(newConfig: Configuration) {
    super.onConfigurationChanged(newConfig)
    PluginManager.onConfigurationChanged(newConfig)
  }
}
