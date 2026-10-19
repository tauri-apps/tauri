// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Application metadata and lifecycle APIs: version and identifier, theme and dock
 * visibility, data stores and exiting the app.
 *
 * This package is also accessible with `window.__TAURI__.app` when [`app.withGlobalTauri`](https://v2.tauri.app/reference/config/#withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * @remarks Only the read-only commands are enabled by `core:app:default`
 * (`allow-version`, `allow-name`, `allow-tauri-version`, `allow-identifier`,
 * `allow-bundle-type`, `allow-supports-multiple-windows`, `allow-register-listener`
 * and `allow-remove-listener`). Every other function in this module documents the
 * permission it needs, which you must add to a capability yourself.
 *
 * @module
 */

import { addPluginListener, invoke, PluginListener } from './core'
import { Image } from './image'
import { Theme } from './window'

/**
 * Identifier type used for data stores on macOS and iOS.
 *
 * Represents a 128-bit identifier, commonly expressed as a 16-byte UUID.
 *
 * @since 2.4.0
 */
export type DataStoreIdentifier = [
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number
]

/**
 * Bundle type of the current application.
 *
 * @see {@linkcode getBundleType}
 *
 * @since 2.7.0
 */
export enum BundleType {
  /** Windows NSIS */
  Nsis = 'nsis',
  /** Windows MSI */
  Msi = 'msi',
  /** Linux Debian package */
  Deb = 'deb',
  /** Linux RPM */
  Rpm = 'rpm',
  /** Linux AppImage */
  AppImage = 'appimage',
  /** macOS app bundle */
  App = 'app'
}

/**
 * Gets the application version.
 * @example
 * ```typescript
 * import { getVersion } from '@tauri-apps/api/app';
 * const appVersion = await getVersion();
 * ```
 *
 * @since 1.0.0
 */
async function getVersion(): Promise<string> {
  return invoke('plugin:app|version')
}

/**
 * Gets the application name.
 * @example
 * ```typescript
 * import { getName } from '@tauri-apps/api/app';
 * const appName = await getName();
 * ```
 *
 * @since 1.0.0
 */
async function getName(): Promise<string> {
  return invoke('plugin:app|name')
}

/**
 * Gets the Tauri framework version used by this application.
 *
 * @example
 * ```typescript
 * import { getTauriVersion } from '@tauri-apps/api/app';
 * const tauriVersion = await getTauriVersion();
 * ```
 *
 * @since 1.0.0
 */
async function getTauriVersion(): Promise<string> {
  return invoke('plugin:app|tauri_version')
}

/**
 * Gets the application identifier.
 * @example
 * ```typescript
 * import { getIdentifier } from '@tauri-apps/api/app';
 * const identifier = await getIdentifier();
 * ```
 *
 * @returns The application identifier as configured in `tauri.conf.json`.
 *
 * @since 2.4.0
 */
async function getIdentifier(): Promise<string> {
  return invoke('plugin:app|identifier')
}

/**
 * Shows the application on macOS. This function does not automatically
 * focus any specific app window.
 *
 * @example
 * ```typescript
 * import { show } from '@tauri-apps/api/app';
 * await show();
 * ```
 *
 * @remarks Requires the `core:app:allow-app-show` permission (not included in `core:app:default`).
 *
 * @since 1.2.0
 */
async function show(): Promise<void> {
  return invoke('plugin:app|app_show')
}

/**
 * Hides the application on macOS.
 *
 * @example
 * ```typescript
 * import { hide } from '@tauri-apps/api/app';
 * await hide();
 * ```
 *
 * @remarks Requires the `core:app:allow-app-hide` permission (not included in `core:app:default`).
 *
 * @since 1.2.0
 */
async function hide(): Promise<void> {
  return invoke('plugin:app|app_hide')
}

/**
 * Fetches the data store identifiers on macOS and iOS.
 *
 * See https://developer.apple.com/documentation/webkit/wkwebsitedatastore for more information.
 *
 * @example
 * ```typescript
 * import { fetchDataStoreIdentifiers } from '@tauri-apps/api/app';
 * const ids = await fetchDataStoreIdentifiers();
 * ```
 *
 * @remarks Requires the `core:app:allow-fetch-data-store-identifiers` permission
 * (not included in `core:app:default`).
 *
 * @since 2.4.0
 */
async function fetchDataStoreIdentifiers(): Promise<DataStoreIdentifier[]> {
  return invoke('plugin:app|fetch_data_store_identifiers')
}

/**
 * Removes the data store with the given identifier.
 *
 * Note that any webview using this data store should be closed before running this API.
 *
 * See https://developer.apple.com/documentation/webkit/wkwebsitedatastore for more information.
 *
 * @example
 * ```typescript
 * import { fetchDataStoreIdentifiers, removeDataStore } from '@tauri-apps/api/app';
 * for (const id of (await fetchDataStoreIdentifiers())) {
 *   await removeDataStore(id);
 * }
 * ```
 *
 * @remarks Requires the `core:app:allow-remove-data-store` permission (not included
 * in `core:app:default`).
 *
 * @since 2.4.0
 */
async function removeDataStore(uuid: DataStoreIdentifier): Promise<void> {
  return invoke('plugin:app|remove_data_store', { uuid })
}

/**
 * Gets the default window icon.
 *
 * @example
 * ```typescript
 * import { defaultWindowIcon } from '@tauri-apps/api/app';
 * const icon = await defaultWindowIcon();
 * ```
 *
 * @remarks Requires the `core:app:allow-default-window-icon` permission (not
 * included in `core:app:default`).
 *
 * @since 2.0.0
 */

async function defaultWindowIcon(): Promise<Image | null> {
  return invoke<number | null>('plugin:app|default_window_icon').then((rid) =>
    rid ? new Image(rid) : null
  )
}

/**
 * Sets the application's theme. Pass in `null` or `undefined` to follow
 * the system theme.
 *
 * @example
 * ```typescript
 * import { setTheme } from '@tauri-apps/api/app';
 * await setTheme('dark');
 * ```
 *
 * #### Platform-specific
 *
 * - **iOS / Android:** Unsupported.
 *
 * @remarks Requires the `core:app:allow-set-app-theme` permission (not included in
 * `core:app:default`).
 *
 * @since 2.0.0
 */
async function setTheme(theme?: Theme | null): Promise<void> {
  return invoke('plugin:app|set_app_theme', { theme })
}

/**
 * Sets the dock visibility for the application on macOS.
 *
 * @param visible - Whether the dock should be visible or not.
 *
 * @example
 * ```typescript
 * import { setDockVisibility } from '@tauri-apps/api/app';
 * await setDockVisibility(false);
 * ```
 *
 * @remarks Requires the `core:app:allow-set-dock-visibility` permission (not
 * included in `core:app:default`).
 *
 * @since 2.5.0
 */
async function setDockVisibility(visible: boolean): Promise<void> {
  return invoke('plugin:app|set_dock_visibility', { visible })
}

/**
 * Gets the application bundle type.
 *
 * @example
 * ```typescript
 * import { getBundleType } from '@tauri-apps/api/app';
 * const type = await getBundleType();
 * ```
 *
 * @since 2.5.0
 */
async function getBundleType(): Promise<BundleType> {
  return invoke('plugin:app|bundle_type')
}

/**
 * Payload for the onBackButtonPress event.
 *
 * @since 2.9.0
 */
type OnBackButtonPressPayload = {
  /** Whether the webview canGoBack property is true. */
  canGoBack: boolean
}

/**
 * Listens to the Android hardware/gesture back button.
 *
 * Registering a handler takes over the default behavior, so the app no longer
 * navigates back or closes on its own: decide what to do inside the handler,
 * using `payload.canGoBack` to know whether the webview has history to go back to.
 *
 * #### Platform-specific
 *
 * - **Android:** Supported.
 * - **Windows / Linux / macOS / iOS:** Unsupported, the handler is never called.
 *
 * @example
 * ```typescript
 * import { onBackButtonPress } from '@tauri-apps/api/app';
 * import { exit } from '@tauri-apps/api/app';
 *
 * const listener = await onBackButtonPress(({ canGoBack }) => {
 *   if (canGoBack) {
 *     window.history.back();
 *   } else {
 *     void exit(0);
 *   }
 * });
 *
 * // stop handling the back button
 * await listener.unregister();
 * ```
 *
 * @param handler Called on every back button press.
 * @returns A listener handle, call `unregister()` on it to restore the default behavior.
 *
 * @since 2.9.0
 */
async function onBackButtonPress(
  handler: (payload: OnBackButtonPressPayload) => void
): Promise<PluginListener> {
  return addPluginListener<OnBackButtonPressPayload>(
    'app',
    'back-button',
    handler
  )
}

/**
 * Whether the current platform can show more than one window at a time.
 *
 * Use it to hide or disable multi-window features on platforms where creating a
 * second window is not possible.
 *
 * #### Platform-specific
 *
 * - **Windows / Linux / macOS:** Always `true`.
 * - **Android:** `true` on API level 32 (Android 12L) and above.
 * - **iOS:** Reflects [`UIApplication.supportsMultipleScenes`](https://developer.apple.com/documentation/uikit/uiapplication/supportsmultiplescenes),
 *   so it is `true` on iPadOS and `false` on iPhone.
 *
 * See the [mobile multiwindow guide](https://tauri.app/learn/mobile-multiwindow/)
 * for how windows behave on mobile.
 *
 * @example
 * ```typescript
 * import { supportsMultipleWindows } from '@tauri-apps/api/app';
 * import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
 *
 * if (await supportsMultipleWindows()) {
 *   new WebviewWindow('settings', { url: '/settings' });
 * }
 * ```
 *
 * @since 2.11.0
 */
async function supportsMultipleWindows(): Promise<boolean> {
  return invoke('plugin:app|supports_multiple_windows')
}

/**
 * Exits the app with the given exit code.
 *
 * This is the same as the `exit` function of the `@tauri-apps/plugin-process` plugin,
 * but does not require a plugin to be installed.
 *
 * #### Platform-specific
 *
 * - **Android**: The activity is finished instead of the process being killed,
 *   so the app closes with the system transition; `code` is ignored.
 *
 * @example
 * ```typescript
 * import { exit } from '@tauri-apps/api/app';
 * await exit(1);
 * ```
 *
 * @param code The exit code to use. Defaults to `0`.
 * @returns A promise indicating the success or failure of the operation.
 *
 * @remarks Requires the `core:app:allow-exit` permission (not included in
 * `core:app:default`).
 *
 * @since 2.12.0
 */
async function exit(code = 0): Promise<void> {
  return invoke('plugin:app|exit', { code })
}

export {
  getName,
  getVersion,
  getTauriVersion,
  getIdentifier,
  show,
  hide,
  defaultWindowIcon,
  setTheme,
  fetchDataStoreIdentifiers,
  removeDataStore,
  setDockVisibility,
  getBundleType,
  type OnBackButtonPressPayload,
  onBackButtonPress,
  supportsMultipleWindows,
  exit
}
