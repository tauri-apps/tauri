// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const ms = require('ms')

let prevTime

module.exports = (banner) => {
  return (msg) => {
    const curr = +new Date()
    const diff = curr - (prevTime || curr)

    prevTime = curr

    if (msg) {
      console.log(
        // eslint-disable-next-line @typescript-eslint/restrict-template-expressions, @typescript-eslint/no-unsafe-call
        ` ${String(banner)} ${msg} ${`+${ms(diff)}`}`
      )
    } else {
      console.log()
    }
  }
}
