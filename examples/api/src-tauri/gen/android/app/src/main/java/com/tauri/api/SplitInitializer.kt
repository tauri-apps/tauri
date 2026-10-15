package com.tauri.api

import android.content.Context
import androidx.startup.Initializer
import androidx.window.embedding.RuleController

// Registers the activity embedding rules at startup (see the androidx.startup provider in AndroidManifest.xml).
class SplitInitializer : Initializer<RuleController> {
  override fun create(context: Context): RuleController {
    return RuleController.getInstance(context).apply {
      setRules(RuleController.parseRules(context, R.xml.main_split_config))
    }
  }

  override fun dependencies(): List<Class<out Initializer<*>>> = emptyList()
}
