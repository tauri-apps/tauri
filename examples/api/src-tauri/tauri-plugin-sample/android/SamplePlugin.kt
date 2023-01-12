package com.tauri.api

import android.webkit.WebView

class SamplePlugin: Plugin() {
  override fun load(webView: WebView) {
    println("loadddd!!!")
  }

  @PluginMethod
  fun run(call: PluginCall) {
    println("running")
    call.resolve()
  }
}
