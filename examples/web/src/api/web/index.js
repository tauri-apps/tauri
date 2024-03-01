// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import init, * as wasm from 'wasm'

function initialize() {
  return init('wasm/wasm_bg.wasm')
}

export const NAME = 'WEB'

/**
 * Greets someone.
 * @param {string} name
 * @returns
 */
export async function greet(name) {
  return initialize().then(() => wasm.greet(name))
}
