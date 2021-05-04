// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'

/**
 * Exits immediately with the given `exitCode`.
 *
 * @param exitCode The exit code to use
 * @returns
 */
async function exit(exitCode: number = 0): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'Process',
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
    __tauriModule: 'Process',
    message: {
      cmd: 'relaunch'
    }
  })
}

export { exit, relaunch }
