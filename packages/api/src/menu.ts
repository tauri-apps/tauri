// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Build native application, window and context menus.
 *
 * A {@linkcode Menu} holds items ({@linkcode MenuItem}, {@linkcode CheckMenuItem},
 * {@linkcode IconMenuItem}, {@linkcode PredefinedMenuItem}) and {@linkcode Submenu}s.
 * Use `Menu.setAsAppMenu()` for the macOS application menu, `Menu.setAsWindowMenu()`
 * for the Windows/Linux window menu bar, or `Menu.popup()` for a context menu.
 *
 * Menus live on the Rust side, the frontend only holds handles to them, so keep a
 * reference to a menu for as long as it is in use.
 *
 * This package is also accessible with `window.__TAURI__.menu` when [`app.withGlobalTauri`](https://v2.tauri.app/reference/config/#withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * @remarks All commands used by this module are part of the `core:menu:default`
 * permission set, which is enabled by default, so no extra capability
 * configuration is needed. Menu item icons additionally require the `image-png` /
 * `image-ico` Cargo features of the `tauri` crate.
 *
 * @module
 */

export * from './menu/submenu'
export * from './menu/menuItem'
export * from './menu/menu'
export * from './menu/checkMenuItem'
export * from './menu/iconMenuItem'
export * from './menu/predefinedMenuItem'
