package app.tauri.plugin

import app.tauri.Permission

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
