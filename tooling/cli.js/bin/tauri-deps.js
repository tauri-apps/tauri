// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import {
  installDependencies,
  updateDependencies
} from '../dist/api/dependency-manager.js'

async function run() {
  const choice = process.argv[2]
  if (choice === 'install') {
    await installDependencies()
  } else if (choice === 'update') {
    await updateDependencies()
  } else {
    console.log(`
    Description
      Tauri dependency management script
    Usage
      $ tauri deps [install|update]
  `)
  }
}

run()
