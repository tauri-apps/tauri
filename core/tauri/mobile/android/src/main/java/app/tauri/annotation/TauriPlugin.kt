// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.annotation

import app.tauri.annotation.Permission

/**
 * Base annotation for all Plugins
 */
@Retention(AnnotationRetention.RUNTIME)
annotation class TauriPlugin(
  /**
   * Permissions this plugin needs, in order to make permission requests
   * easy if the plugin only needs basic permission prompting
   */
  val permissions: Array<Permission> = []
)
