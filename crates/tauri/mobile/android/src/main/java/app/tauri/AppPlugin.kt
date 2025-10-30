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

  override fun load(webView: WebView) {
    this.webView = webView
  }

  init {
    val callback = object : OnBackPressedCallback(true) {
      override fun handleOnBackPressed() {
        if (!hasListener(BACK_BUTTON_EVENT)) {
          if (this@AppPlugin.webView?.canGoBack() == true) {
            this@AppPlugin.webView!!.goBack()
          } else {
            this.isEnabled = false
            this@AppPlugin.activity.onBackPressed()
            this.isEnabled = true
          }
        } else {
          val data = JSObject().apply {
            put("canGoBack", this@AppPlugin.webView?.canGoBack() ?: false)
          }
          trigger(BACK_BUTTON_EVENT, data)
        }
      }
    }
    (activity as AppCompatActivity).onBackPressedDispatcher.addCallback(activity, callback)
  }

  @Command
  fun exit(invoke: Invoke) {
    invoke.resolve()
    activity.finish()
  }
}
