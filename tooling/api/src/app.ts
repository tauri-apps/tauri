// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Get application metadata.
 *
 * This package is also accessible with `window.__TAURI__.app` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
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
  return invokeTauriCommand<string>({
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
  return invokeTauriCommand<string>({
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
  return invokeTauriCommand<string>({
    __tauriModule: 'App',
    message: {
      cmd: 'getTauriVersion'
    }
  })
}

export { getName, getVersion, getTauriVersion }
