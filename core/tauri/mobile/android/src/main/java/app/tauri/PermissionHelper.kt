// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri

// taken from https://github.com/ionic-team/capacitor/blob/6658bca41e78239347e458175b14ca8bd5c1d6e8/android/capacitor/src/main/java/com/getcapacitor/PermissionHelper.java

import android.content.Context;
import android.content.pm.PackageManager;
import android.os.Build;
import androidx.core.app.ActivityCompat;
import java.util.ArrayList;

object PermissionHelper {
  /**
   * Checks if a list of given permissions are all granted by the user
   *
   * @param permissions Permissions to check.
   * @return True if all permissions are granted, false if at least one is not.
   */
  fun hasPermissions(context: Context?, permissions: Array<String>): Boolean {
    for (perm in permissions) {
      if (ActivityCompat.checkSelfPermission(
          context!!,
          perm
        ) != PackageManager.PERMISSION_GRANTED
      ) {
        return false
      }
    }
    return true
  }

  /**
   * Check whether the given permission has been defined in the AndroidManifest.xml
   *
   * @param permission A permission to check.
   * @return True if the permission has been defined in the Manifest, false if not.
   */
  fun hasDefinedPermission(context: Context, permission: String): Boolean {
    var hasPermission = false
    val requestedPermissions = getManifestPermissions(context)
    if (requestedPermissions != null && requestedPermissions.isNotEmpty()) {
      val requestedPermissionsList = listOf(*requestedPermissions)
      val requestedPermissionsArrayList = ArrayList(requestedPermissionsList)
      if (requestedPermissionsArrayList.contains(permission)) {
        hasPermission = true
      }
    }
    return hasPermission
  }

  /**
   * Check whether all of the given permissions have been defined in the AndroidManifest.xml
   * @param context the app context
   * @param permissions a list of permissions
   * @return true only if all permissions are defined in the AndroidManifest.xml
   */
  fun hasDefinedPermissions(context: Context, permissions: Array<String>): Boolean {
    for (permission in permissions) {
      if (!hasDefinedPermission(context, permission)) {
        return false
      }
    }
    return true
  }

  /**
   * Get the permissions defined in AndroidManifest.xml
   *
   * @return The permissions defined in AndroidManifest.xml
   */
  private fun getManifestPermissions(context: Context): Array<String>? {
    var requestedPermissions: Array<String>? = null
    try {
      val pm = context.packageManager
      val packageInfo = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        pm.getPackageInfo(context.packageName, PackageManager.PackageInfoFlags.of(PackageManager.GET_PERMISSIONS.toLong()))
      } else {
        @Suppress("DEPRECATION")
        pm.getPackageInfo(context.packageName, PackageManager.GET_PERMISSIONS)
      }
      if (packageInfo != null) {
        requestedPermissions = packageInfo.requestedPermissions
      }
    } catch (_: Exception) {
    }
    return requestedPermissions
  }

  /**
   * Given a list of permissions, return a new list with the ones not present in AndroidManifest.xml
   *
   * @param neededPermissions The permissions needed.
   * @return The permissions not present in AndroidManifest.xml
   */
  fun getUndefinedPermissions(context: Context, neededPermissions: Array<String>): Array<String> {
    val undefinedPermissions = ArrayList<String>()
    val requestedPermissions = getManifestPermissions(context)
    if (!requestedPermissions.isNullOrEmpty()) {
      val requestedPermissionsList = listOf(*requestedPermissions)
      val requestedPermissionsArrayList = ArrayList(requestedPermissionsList)
      for (permission in neededPermissions) {
        if (!requestedPermissionsArrayList.contains(permission)) {
          undefinedPermissions.add(permission)
        }
      }
      return undefinedPermissions.toTypedArray()
    }
    return neededPermissions
  }
}
