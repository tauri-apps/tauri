// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri

import android.app.Activity
import android.os.Environment
import app.tauri.annotation.Command
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

    @Command
    fun getAudioDir(invoke: Invoke) {
        resolvePath(invoke, activity.getExternalFilesDir(Environment.DIRECTORY_MUSIC)?.absolutePath)
    }

    @Command
    fun getExternalCacheDir(invoke: Invoke) {
        resolvePath(invoke, activity.externalCacheDir?.absolutePath)
    }

    @Command
    fun getConfigDir(invoke: Invoke) {
        resolvePath(invoke, activity.dataDir.absolutePath)
    }

    @Command
    fun getDataDir(invoke: Invoke) {
        resolvePath(invoke, activity.dataDir.absolutePath)
    }

    @Command
    fun getDocumentDir(invoke: Invoke) {
        resolvePath(invoke, activity.getExternalFilesDir(Environment.DIRECTORY_DOCUMENTS)?.absolutePath)
    }

    @Command
    fun getDownloadDir(invoke: Invoke) {
        resolvePath(invoke, activity.getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS)?.absolutePath)
    }

    @Command
    fun getPictureDir(invoke: Invoke) {
        resolvePath(invoke, activity.getExternalFilesDir(Environment.DIRECTORY_PICTURES)?.absolutePath)
    }

    @Command
    fun getPublicDir(invoke: Invoke) {
        resolvePath(invoke, activity.getExternalFilesDir(Environment.DIRECTORY_DCIM)?.absolutePath)
    }

    @Command
    fun getVideoDir(invoke: Invoke) {
        resolvePath(invoke, activity.externalCacheDir?.absolutePath)
    }

    @Command
    fun getResourcesDir(invoke: Invoke) {
        // TODO
        resolvePath(invoke, activity.cacheDir.absolutePath)
    }

    @Command
    fun getCacheDir(invoke: Invoke) {
        resolvePath(invoke, activity.cacheDir.absolutePath)
    }
}
