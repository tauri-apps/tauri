// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Read and write to the system clipboard.
 *
 * This package is also accessible with `window.__TAURI__.clipboard` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * The APIs must be added to [`tauri.allowlist.clipboard`](https://tauri.app/v1/api/config/#allowlistconfig.clipboard) in `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "clipboard": {
 *         "all": true, // enable all Clipboard APIs
 *         "writeText": true,
 *         "readText": true
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
 * Writes plain text to the clipboard.
 * @example
 * ```typescript
 * import { writeText, readText } from '@tauri-apps/api/clipboard';
 * await writeText('Tauri is awesome!');
 * assert(await readText(), 'Tauri is awesome!');
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0.
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
 * @example
 * ```typescript
 * import { readText } from '@tauri-apps/api/clipboard';
 * const clipboardText = await readText();
 * ```
 * @since 1.0.0.
 */
async function readText(): Promise<string | null> {
  return invokeTauriCommand({
    __tauriModule: 'Clipboard',
    message: {
      cmd: 'readText',
      // if data is not set, `serde` will ignore the custom deserializer
      // that is set when the API is not allowlisted
      data: null
    }
  })
}

export { writeText, readText }
