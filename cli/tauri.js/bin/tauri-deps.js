// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

async function run() {
  const {
    installDependencies,
    updateDependencies
  } = require('../dist/api/dependency-manager')

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
