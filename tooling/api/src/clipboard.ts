// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Read and write to the system clipboard.
 *
 * This package is also accessible with `window.__TAURI__.clipboard` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'

/**
 * Writes a plain text to the clipboard.
 *
 * @returns A promise indicating the success or failure of the operation.
 */
async function writeText(text: string): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'Clipboard',
    message: {
      cmd: 'writeText',
      data: text
    }
  })
}

/**
 * Gets the clipboard content as plain text.
 *
 * @returns A promise resolving to the clipboard content as plain text.
 */
async function readText(): Promise<string | null> {
  return invokeTauriCommand({
    __tauriModule: 'Clipboard',
    message: {
      cmd: 'readText'
    }
  })
}

export { writeText, readText }
