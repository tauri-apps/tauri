// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri

import android.app.Activity
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback
import androidx.appcompat.app.AppCompatActivity
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject

@TauriPlugin
class AppPlugin(private val activity: Activity): Plugin(activity) {
  private val BACK_BUTTON_EVENT = "back-button"

  private var webView: WebView? = null

  // Enabled only while the webview can go back or the app listens for `back-button`, so
  // that otherwise the system handles back and can play its predictive back animation.
  private val callback = object : OnBackPressedCallback(false) {
    override fun handleOnBackPressed() {
      if (!hasListener(BACK_BUTTON_EVENT)) {
        if (this@AppPlugin.webView?.canGoBack() == true) {
          this@AppPlugin.webView!!.goBack()
        } else {
          this.isEnabled = false
          this@AppPlugin.activity.onBackPressed()
          updateCallback()
        }
      } else {
        val data = JSObject().apply {
          put("canGoBack", this@AppPlugin.webView?.canGoBack() ?: false)
        }
        trigger(BACK_BUTTON_EVENT, data)
      }
    }
  }

  init {
    (activity as AppCompatActivity).onBackPressedDispatcher.addCallback(activity, callback)
  }

  override fun load(webView: WebView) {
    this.webView = webView
    updateCallback()
  }

  override fun onWebViewHistoryChanged(webView: WebView) {
    updateCallback()
  }

  @Command
  override fun registerListener(invoke: Invoke) {
    super.registerListener(invoke)
    updateCallback()
  }

  @Command
  override fun removeListener(invoke: Invoke) {
    super.removeListener(invoke)
    updateCallback()
  }

  private fun updateCallback() {
    activity.runOnUiThread {
      callback.isEnabled = hasListener(BACK_BUTTON_EVENT) || webView?.canGoBack() == true    }
  }

  // Called by the Rust side when the app exits so the activity (and any other activity
  // of the task, i.e. other windows) is closed gracefully instead of the process being killed
  @Command
  fun exit(invoke: Invoke) {
    invoke.resolve()
    activity.finishAffinity()
  }
}
