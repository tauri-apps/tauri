// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from '@tauri-apps/api/core'

export const NAME = 'Tauri'

/**
 * Greets someone.
 * @param {string} name
 * @returns
 */
export async function greet(name) {
  return invoke('greet', { name })
}
