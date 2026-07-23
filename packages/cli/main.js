// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const { run, logError } = require('./index')

module.exports.run = (args, binName) => {
  return new Promise((resolve, reject) => {
    run(args, binName, (error, res) => {
      if (error) {
        reject(error)
      } else {
        resolve(res)
      }
    })
  })
}

module.exports.logError = logError
