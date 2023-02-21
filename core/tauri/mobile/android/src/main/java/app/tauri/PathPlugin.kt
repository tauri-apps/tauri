// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri

import android.app.Activity
import android.os.Environment
import app.tauri.annotation.PluginMethod
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject

@TauriPlugin
class PathPlugin(private val activity: Activity): Plugin(activity) {
    private fun resolvePath(invoke: Invoke, path: String?) {
        val obj = JSObject()
        obj.put("path", path)
        invoke.resolve(obj)
    }

    @PluginMethod
    fun getAudioDir(invoke: Invoke) {
        resolvePath(invoke, activity.getExternalFilesDir(Environment.DIRECTORY_MUSIC)?.absolutePath)
    }

    @PluginMethod
    fun getExternalCacheDir(invoke: Invoke) {
        resolvePath(invoke, activity.externalCacheDir?.absolutePath)
    }

    @PluginMethod
    fun getConfigDir(invoke: Invoke) {
        resolvePath(invoke, activity.dataDir.absolutePath)
    }

    @PluginMethod
    fun getDataDir(invoke: Invoke) {
        resolvePath(invoke, activity.dataDir.absolutePath)
    }

    @PluginMethod
    fun getDocumentDir(invoke: Invoke) {
        resolvePath(invoke, activity.getExternalFilesDir(Environment.DIRECTORY_DOCUMENTS)?.absolutePath)
    }

    @PluginMethod
    fun getDownloadDir(invoke: Invoke) {
        resolvePath(invoke, activity.getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS)?.absolutePath)
    }

    @PluginMethod
    fun getPictureDir(invoke: Invoke) {
        resolvePath(invoke, activity.getExternalFilesDir(Environment.DIRECTORY_PICTURES)?.absolutePath)
    }

    @PluginMethod
    fun getPublicDir(invoke: Invoke) {
        resolvePath(invoke, activity.getExternalFilesDir(Environment.DIRECTORY_DCIM)?.absolutePath)
    }

    @PluginMethod
    fun getVideoDir(invoke: Invoke) {
        resolvePath(invoke, activity.externalCacheDir?.absolutePath)
    }

    @PluginMethod
    fun getResourcesDir(invoke: Invoke) {
        // TODO
        resolvePath(invoke, activity.cacheDir.absolutePath)
    }

    @PluginMethod
    fun getCacheDir(invoke: Invoke) {
        resolvePath(invoke, activity.cacheDir.absolutePath)
    }
}
