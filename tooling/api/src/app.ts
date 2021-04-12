// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'

/**
 * Gets the application version.
 *
 * @returns A promise resolving to application version.
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
 * Gets the application name.
 *
 * @returns A promise resolving to application name.
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
 * Gets the tauri version.
 *
 * @returns A promise resolving to tauri version.
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
 * Exits immediately with the given `exitCode`.
 *
 * @param exitCode The exit code to use
 * @returns
 */
async function exit(exitCode: number = 0): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'App',
    mainThread: true,
    message: {
      cmd: 'exit',
      exitCode
    }
  })
}

/**
 * Exits the current instance of the app then relaunches it.
 *
 * @returns
 */
async function relaunch(): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'App',
    mainThread: true,
    message: {
      cmd: 'relaunch'
    }
  })
}

export { getName, getVersion, getTauriVersion, relaunch, exit }
