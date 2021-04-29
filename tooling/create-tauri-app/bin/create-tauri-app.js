#!/usr/bin/env node
// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const { createTauriApp } = require('../dist/')

createTauriApp(process.argv.slice(2)).catch((err) => {
  console.error(err)
})
