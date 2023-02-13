// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Get application metadata.
 *
 * This package is also accessible with `window.__TAURI__.app` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * The APIs must be added to [`tauri.allowlist.app`](https://tauri.app/v1/api/config/#allowlistconfig.app) in `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "app": {
 *         "all": true, // enable all app APIs
 *         "show": true,
 *         "hide": true
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 *
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'

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
  return invokeTauriCommand({
    __tauriModule: 'App',
    message: {
      cmd: 'getAppVersion'
    }
  })
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
  return invokeTauriCommand({
    __tauriModule: 'App',
    message: {
      cmd: 'getAppName'
    }
  })
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
  return invokeTauriCommand({
    __tauriModule: 'App',
    message: {
      cmd: 'getTauriVersion'
    }
  })
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
  return invokeTauriCommand({
    __tauriModule: 'App',
    message: {
      cmd: 'show'
    }
  })
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
  return invokeTauriCommand({
    __tauriModule: 'App',
    message: {
      cmd: 'hide'
    }
  })
}

export { getName, getVersion, getTauriVersion, show, hide }
