// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri

import android.app.Activity
import android.database.Cursor
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.OpenableColumns
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject

const val TAURI_ASSETS_DIRECTORY_URI = "asset://localhost/"

@InvokeArg
class GetFileNameFromUriArgs {
  lateinit var uri: String
}

@TauriPlugin
class PathPlugin(private val activity: Activity): Plugin(activity) {
    private fun resolvePath(invoke: Invoke, path: String?) {
        val obj = JSObject()
        obj.put("path", path)
        invoke.resolve(obj)
    }

    @Command
    fun getFileNameFromUri(invoke: Invoke) {
      val args = invoke.parseArgs(GetFileNameFromUriArgs::class.java)
      val name = getRealNameFromURI(activity, Uri.parse(args.uri))
      val res = JSObject()
      res.put("name", name)
      invoke.resolve(res)
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
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
        resolvePath(invoke, activity.dataDir.absolutePath)
      } else {
        resolvePath(invoke, activity.applicationInfo.dataDir)
      }
    }

    @Command
    fun getDataDir(invoke: Invoke) {
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
        resolvePath(invoke, activity.dataDir.absolutePath)
      } else {
        resolvePath(invoke, activity.applicationInfo.dataDir)
      }
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
        resolvePath(invoke, TAURI_ASSETS_DIRECTORY_URI)
    }

    @Command
    fun getCacheDir(invoke: Invoke) {
        resolvePath(invoke, activity.cacheDir.absolutePath)
    }

    @Command
    fun getHomeDir(invoke: Invoke) {
        resolvePath(invoke, Environment.getExternalStorageDirectory().absolutePath)
    }
}

fun getRealNameFromURI(activity: Activity, contentUri: Uri): String? {
    var cursor: Cursor? = null
    try {
        val projection = arrayOf(OpenableColumns.DISPLAY_NAME)
        cursor = activity.contentResolver.query(contentUri, projection, null, null, null)

        cursor?.let {
            val columnIndex = it.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            if (it.moveToFirst()) {
                return it.getString(columnIndex)
            }
        }
    } catch (e: Exception) {
        Logger.error("failed to get real name from URI $e")
    } finally {
        cursor?.close()
    }

    return null // Return null if no file name could be resolved
}
