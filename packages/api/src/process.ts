// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from './core'

/**
 * Process-related APIs.
 *
 * @module
 */

/**
 * Exits the app.
 *
 * @param exitCode - The exit code to use. Defaults to `0`.
 *
 * @example
 * ```typescript
 * import { exit } from '@tauri-apps/api/process';
 * await exit();
 * ```
 *
 * @since 2.10.0
 */
async function exit(exitCode?: number): Promise<void> {
  const payload = exitCode === undefined ? {} : { exitCode }
  return invoke('plugin:app|exit', payload)
}

export { exit }
