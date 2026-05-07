// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { logError, run as bindingRun } from './cli.wasi-browser.js'

export function run(args, binName) {
  return new Promise((resolve, reject) => {
    bindingRun(args, binName, (error, res) => {
      if (error) {
        reject(error)
      } else {
        resolve(res)
      }
    })
  })
}

export { logError }
