export * from './menu/submenu'
export * from './menu/menuItem'
export * from './menu/menu'
export * from './menu/checkMenuItem'
export * from './menu/iconMenuItem'
export * from './menu/predefinedMenuItem'

/**
 * Menu types and utilities.
 *
 * This package is also accessible with `window.__TAURI__.menu` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 * @module
 */

/** Describes a menu event emitted when a menu item is activated */
export interface MenuEvent {
  /** Id of the menu item that triggered this event */
  id: string
}
