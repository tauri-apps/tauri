// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Get application metadata.
 *
 * This package is also accessible with `window.__TAURI__.app` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'

/**
 * Gets the application version.
 *
 * @returns A promise resolving to the application version.
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
 *
 * @returns A promise resolving to application name.
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
 * Gets the tauri version.
 *
 * @returns A promise resolving to tauri version.
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
