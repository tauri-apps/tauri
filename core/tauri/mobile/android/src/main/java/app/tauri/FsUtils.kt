// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri

import android.content.ContentUris
import android.content.Context
import android.content.res.AssetManager
import android.database.Cursor
import android.net.Uri
import android.os.Environment
import android.provider.DocumentsContract
import android.provider.MediaStore
import android.provider.OpenableColumns
import java.io.File
import java.io.FileOutputStream
import kotlin.math.min

internal class FsUtils {
  companion object {
    fun readAsset(assetManager: AssetManager, fileName: String): String {
      assetManager.open(fileName).bufferedReader().use {
        return it.readText()
      }
    }

    fun getFileUrlForUri(context: Context, uri: Uri): String? {
      // DocumentProvider
      if (DocumentsContract.isDocumentUri(context, uri)) {
        // ExternalStorageProvider
        if (isExternalStorageDocument(uri)) {
          val docId: String = DocumentsContract.getDocumentId(uri)
          val split = docId.split(":".toRegex()).dropLastWhile { it.isEmpty() }
            .toTypedArray()
          val type = split[0]
          if ("primary".equals(type, ignoreCase = true)) {
            return legacyPrimaryPath(split[1])
          } else {
            val splitIndex = docId.indexOf(':', 1)
            val tag = docId.substring(0, splitIndex)
            val path = docId.substring(splitIndex + 1)
            val nonPrimaryVolume = getPathToNonPrimaryVolume(context, tag)
            if (nonPrimaryVolume != null) {
              val result = "$nonPrimaryVolume/$path"
              val file = File(result)
              return if (file.exists() && file.canRead()) {
                result
              } else null
            }
          }
        } else if (isDownloadsDocument(uri)) {
          val id: String = DocumentsContract.getDocumentId(uri)
          val contentUri: Uri = ContentUris.withAppendedId(
            Uri.parse("content://downloads/public_downloads"),
            java.lang.Long.valueOf(id)
          )
          return getDataColumn(context, contentUri, null, null)
        } else if (isMediaDocument(uri)) {
          val docId: String = DocumentsContract.getDocumentId(uri)
          val split = docId.split(":".toRegex()).dropLastWhile { it.isEmpty() }
            .toTypedArray()
          val type = split[0]
          var contentUri: Uri? = null
          when (type) {
            "image" -> {
              contentUri = MediaStore.Images.Media.EXTERNAL_CONTENT_URI
            }
            "video" -> {
              contentUri = MediaStore.Video.Media.EXTERNAL_CONTENT_URI
            }
            "audio" -> {
              contentUri = MediaStore.Audio.Media.EXTERNAL_CONTENT_URI
            }
          }
          val selection = "_id=?"
          val selectionArgs = arrayOf(split[1])
          if (contentUri != null) {
            return getDataColumn(context, contentUri, selection, selectionArgs)
          }
        }
      } else if ("content".equals(uri.scheme, ignoreCase = true)) {
        // Return the remote address
        return if (isGooglePhotosUri(uri)) uri.lastPathSegment else getDataColumn(
          context,
          uri,
          null,
          null
        )
      } else if ("file".equals(uri.scheme, ignoreCase = true)) {
        return uri.path
      }
      return null
    }

    /**
     * Get the value of the data column for this Uri. This is useful for
     * MediaStore Uris, and other file-based ContentProviders.
     *
     * @param context The context.
     * @param uri The Uri to query.
     * @param selection (Optional) Filter used in the query.
     * @param selectionArgs (Optional) Selection arguments used in the query.
     * @return The value of the _data column, which is typically a file path.
     */
    private fun getDataColumn(
      context: Context,
      uri: Uri,
      selection: String?,
      selectionArgs: Array<String>?
    ): String? {
      var path: String? = null
      var cursor: Cursor? = null
      val column = "_data"
      val projection = arrayOf(column)
      try {
        cursor = context.contentResolver.query(uri, projection, selection, selectionArgs, null)
        if (cursor != null && cursor.moveToFirst()) {
          val index = cursor.getColumnIndexOrThrow(column)
          path = cursor.getString(index)
        }
      } catch (ex: IllegalArgumentException) {
        return getCopyFilePath(uri, context)
      } finally {
        cursor?.close()
      }
      return path ?: getCopyFilePath(uri, context)
    }

    private fun getCopyFilePath(uri: Uri, context: Context): String? {
      val cursor = context.contentResolver.query(uri, null, null, null, null)!!
      val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
      cursor.moveToFirst()
      val name = cursor.getString(nameIndex)
      val file = File(context.filesDir, name)
      try {
        val inputStream = context.contentResolver.openInputStream(uri)
        val outputStream = FileOutputStream(file)
        var read: Int
        val maxBufferSize = 1024 * 1024
        val bufferSize = min(inputStream!!.available(), maxBufferSize)
        val buffers = ByteArray(bufferSize)
        while (inputStream.read(buffers).also { read = it } != -1) {
          outputStream.write(buffers, 0, read)
        }
        inputStream.close()
        outputStream.close()
      } catch (e: Exception) {
        return null
      } finally {
        cursor.close()
      }
      return file.path
    }

    private fun legacyPrimaryPath(pathPart: String): String {
      return Environment.getExternalStorageDirectory().toString() + "/" + pathPart
    }

    /**
     * @param uri The Uri to check.
     * @return Whether the Uri authority is ExternalStorageProvider.
     */
    private fun isExternalStorageDocument(uri: Uri): Boolean {
      return "com.android.externalstorage.documents" == uri.authority
    }

    /**
     * @param uri The Uri to check.
     * @return Whether the Uri authority is DownloadsProvider.
     */
    private fun isDownloadsDocument(uri: Uri): Boolean {
      return "com.android.providers.downloads.documents" == uri.authority
    }

    /**
     * @param uri The Uri to check.
     * @return Whether the Uri authority is MediaProvider.
     */
    private fun isMediaDocument(uri: Uri): Boolean {
      return "com.android.providers.media.documents" == uri.authority
    }

    /**
     * @param uri The Uri to check.
     * @return Whether the Uri authority is Google Photos.
     */
    private fun isGooglePhotosUri(uri: Uri): Boolean {
      return "com.google.android.apps.photos.content" == uri.authority
    }

    private fun getPathToNonPrimaryVolume(context: Context, tag: String): String? {
      val volumes = context.externalCacheDirs
      if (volumes != null) {
        for (volume in volumes) {
          if (volume != null) {
            val path = volume.absolutePath
            val index = path.indexOf(tag)
            if (index != -1) {
              return path.substring(0, index) + tag
            }
          }
        }
      }
      return null
    }
  }
}
