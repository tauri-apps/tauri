// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const { write, onMessage } = require('./communication')

onMessage((line) => {
  write(`read ${line}`)
})

setInterval(() => {
  write(`[${new Date().toLocaleTimeString()}] new message`)
}, 500)
