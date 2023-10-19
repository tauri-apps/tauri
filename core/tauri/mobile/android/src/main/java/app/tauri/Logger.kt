// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri

// taken from https://github.com/ionic-team/capacitor/blob/6658bca41e78239347e458175b14ca8bd5c1d6e8/android/capacitor/src/main/java/com/getcapacitor/Logger.java

import android.text.TextUtils;
import android.util.Log;

class Logger {
  companion object {
    private const val LOG_TAG_CORE = "Tauri"

    fun tags(vararg subtags: String): String {
      return if (subtags.isNotEmpty()) {
        LOG_TAG_CORE + "/" + TextUtils.join("/", subtags)
      } else LOG_TAG_CORE
    }

    fun verbose(message: String) {
      verbose(LOG_TAG_CORE, message)
    }

    fun verbose(tag: String, message: String) {
      if (!shouldLog()) {
        return
      }
      Log.v(tag, message)
    }

    fun debug(message: String) {
      debug(LOG_TAG_CORE, message)
    }

    fun debug(tag: String, message: String) {
      if (!shouldLog()) {
        return
      }
      Log.d(tag, message)
    }

    fun info(message: String) {
      info(LOG_TAG_CORE, message)
    }

    fun info(tag: String, message: String) {
      if (!shouldLog()) {
        return
      }
      Log.i(tag, message)
    }

    fun warn(message: String) {
      warn(LOG_TAG_CORE, message)
    }

    fun warn(tag: String, message: String) {
      if (!shouldLog()) {
        return
      }
      Log.w(tag, message)
    }

    fun error(message: String) {
      error(LOG_TAG_CORE, message, null)
    }

    fun error(message: String, e: Throwable?) {
      error(LOG_TAG_CORE, message, e)
    }

    fun error(tag: String, message: String, e: Throwable?) {
      if (!shouldLog()) {
        return
      }
      Log.e(tag, message, e)
    }

    private fun shouldLog(): Boolean {
      return BuildConfig.DEBUG
    }
  }
}
