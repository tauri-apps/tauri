// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri

import android.app.Activity
import android.app.Application
import android.content.ActivityNotFoundException
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.Settings
import android.view.WindowManager
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback
import androidx.appcompat.app.AppCompatActivity
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import java.lang.ref.WeakReference
import java.util.Collections
import java.util.WeakHashMap

@InvokeArg
internal class SetEnabledArgs {
  var enabled: Boolean = false
}

/**
 * Window flags applied to every activity of the app.
 *
 * Each Tauri window is its own activity, and window flags such as FLAG_SECURE belong to a single
 * activity's window, so the flags are tracked here and applied to the activities that exist when
 * they change and to every activity created afterwards.
 *
 * Process-wide rather than per plugin instance so a new AppPlugin instance does not register
 * the lifecycle callbacks again. Only accessed on the UI thread.
 */
private object AppWindows : Application.ActivityLifecycleCallbacks {
  private var registered = false
  private val activities: MutableSet<Activity> = Collections.newSetFromMap(WeakHashMap())
  private var resumed: WeakReference<Activity>? = null
  private var flags = 0

  fun track(activity: Activity) {
    if (!registered) {
      activity.application.registerActivityLifecycleCallbacks(this)
      registered = true
    }
    if (activities.add(activity) && flags != 0) {
      activity.window.addFlags(flags)
    }
  }

  fun setFlag(flag: Int, enabled: Boolean) {
    flags = if (enabled) flags or flag else flags and flag.inv()
    for (activity in activities.toList()) {
      if (enabled) {
        activity.window.addFlags(flag)
      } else {
        activity.window.clearFlags(flag)
      }
    }
  }

  /** The activity to start other activities from: the resumed one, else any live one. */
  fun launcher(): Activity? {
    val isAlive = { a: Activity -> !a.isFinishing && !a.isDestroyed }
    return resumed?.get()?.takeIf(isAlive) ?: activities.firstOrNull(isAlive)
  }

  override fun onActivityCreated(activity: Activity, savedInstanceState: Bundle?) {
    track(activity)
  }

  override fun onActivityResumed(activity: Activity) {
    track(activity)
    resumed = WeakReference(activity)
  }

  override fun onActivityDestroyed(activity: Activity) {
    activities.remove(activity)
    if (resumed?.get() == activity) {
      resumed = null
    }
  }

  override fun onActivityStarted(activity: Activity) {}
  override fun onActivityPaused(activity: Activity) {}
  override fun onActivityStopped(activity: Activity) {}
  override fun onActivitySaveInstanceState(activity: Activity, outState: Bundle) {}
}

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
    activity.runOnUiThread { AppWindows.track(activity) }
  }

  override fun load(webView: WebView) {
    this.webView = webView
    updateCallback()
  }

  override fun onWebViewHistoryChanged(webView: WebView) {
    updateCallback()
  }

  // Catches windows that already existed before the lifecycle callbacks were registered.
  override fun onResume(activity: AppCompatActivity) {
    AppWindows.track(activity)
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
  fun openSettings(invoke: Invoke) {
    val intent = Intent(
      Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
      Uri.fromParts("package", activity.packageName, null)
    )
    activity.runOnUiThread {
      try {
        val launcher = AppWindows.launcher()
        if (launcher != null) {
          launcher.startActivity(intent)
        } else {
          intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
          activity.applicationContext.startActivity(intent)
        }
        invoke.resolve()
      } catch (e: ActivityNotFoundException) {
        invoke.reject("no activity can open the app settings: ${e.message}")
      }
    }
  }

  @Command
  fun setKeepScreenOn(invoke: Invoke) {
    setWindowFlag(invoke, WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
  }

  @Command
  fun setPrivacyScreen(invoke: Invoke) {
    setWindowFlag(invoke, WindowManager.LayoutParams.FLAG_SECURE)
  }

  private fun setWindowFlag(invoke: Invoke, flag: Int) {
    val args = invoke.parseArgs(SetEnabledArgs::class.java)
    activity.runOnUiThread {
      AppWindows.setFlag(flag, args.enabled)
      invoke.resolve()
    }
  }

  @Command
  fun exit(invoke: Invoke) {
    invoke.resolve()
    activity.finishAffinity()
  }
}
