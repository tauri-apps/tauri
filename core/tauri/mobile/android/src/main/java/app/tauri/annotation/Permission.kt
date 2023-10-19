// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.annotation

@Retention(AnnotationRetention.RUNTIME)
annotation class Permission(
  /**
   * An array of Android permission strings.
   * Eg: {Manifest.permission.ACCESS_COARSE_LOCATION}
   * or {"android.permission.ACCESS_COARSE_LOCATION"}
   */
  val strings: Array<String> = [],
  /**
   * An optional name to use instead of the Android permission string.
   */
  val alias: String = ""
)
