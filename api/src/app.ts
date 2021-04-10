// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'

/**
 * @name getVersion
 * @description Get application version
 * @returns {Promise<string>} Promise resolving to application version
 */
async function getVersion(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'App',
    mainThread: true,
    message: {
      cmd: 'getAppVersion'
    }
  })
}

/**
 * @name getName
 * @description Get application name
 * @returns {Promise<string>} Promise resolving to application name
 */
async function getName(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'App',
    mainThread: true,
    message: {
      cmd: 'getAppName'
    }
  })
}

/**
 * @name getTauriVersion
 * @description Get tauri version
 * @returns {Promise<string>} Promise resolving to tauri version
 */
async function getTauriVersion(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'App',
    mainThread: true,
    message: {
      cmd: 'getTauriVersion'
    }
  })
}

/**
 * @name exit
 * @description Exits immediately with exitCode.
 * @param [exitCode] defaults to 0.
 * @returns {Promise<void>} Application is closing, nothing is returned
 */
async function exit(exitCode: Number = 0): Promise<void> {
  return invokeTauriCommand<string>({
    __tauriModule: 'App',
    mainThread: true,
    message: {
      cmd: 'exit',
      exitCode
    }
  })
}

/**
 * @name relaunch
 * @description Relaunches the app when current instance exits.
 * @returns {Promise<void>} Application is restarting, nothing is returned
 */
async function relaunch(): Promise<void> {
  return invokeTauriCommand<string>({
    __tauriModule: 'App',
    mainThread: true,
    message: {
      cmd: 'relaunch'
    }
  })
}

export { getName, getVersion, getTauriVersion, relaunch, exit }
