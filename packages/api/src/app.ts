// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from './core'
import { Image } from './image'
import { Theme } from './window'

/**
 * Application metadata and related APIs.
 *
 * @module
 */

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
 * Gets the Tauri version.
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
 * Shows the application on macOS. This function does not automatically focus any specific app window.
 *
 * @example
 * ```typescript
 * import { show } from '@tauri-apps/api/app';
 * await show();
 * ```
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
 * @since 1.2.0
 */
async function hide(): Promise<void> {
  return invoke('plugin:app|app_hide')
}

/**
 * Get the default window icon.
 *
 * @example
 * ```typescript
 * import { defaultWindowIcon } from '@tauri-apps/api/app';
 * await defaultWindowIcon();
 * ```
 *
 * @since 2.0.0
 */
async function defaultWindowIcon(): Promise<Image | null> {
  return invoke<number | null>('plugin:app|default_window_icon').then((rid) =>
    rid ? new Image(rid) : null
  )
}

/**
 * Set app's theme, pass in `null` or `undefined` to follow system theme
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
 * @since 2.0.0
 */
async function setTheme(theme?: Theme | null): Promise<void> {
  return invoke('plugin:app|set_app_theme', { theme })
}

export {
  getName,
  getVersion,
  getTauriVersion,
  show,
  hide,
  defaultWindowIcon,
  setTheme
}
