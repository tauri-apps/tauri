#!/usr/bin/env node
// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const cmds = ['icon', 'deps']
const rustCliCmds = ['dev', 'build', 'init', 'info', 'sign']

const cmd = process.argv[2]

/**
 * @description This is the bootstrapper that in turn calls subsequent
 * Tauri Commands
 *
 * @param {string|array} command
 */
const tauri = function (command) {
  if (typeof command === 'object') {
    // technically we just care about an array
    command = command[0]
  }

  if (rustCliCmds.includes(command)) {
    const { runOnRustCli } = require('../dist/helpers/rust-cli')
    if (process.argv && !process.env.test) {
      process.argv.splice(0, 3)
    }
    runOnRustCli(command, process.argv || []).promise.then(() => {
      if (command === 'init') {
        const {
          installDependencies
        } = require('../dist/api/dependency-manager')
        return installDependencies()
      }
    })
    return
  }

  if (
    !command ||
    command === '-h' ||
    command === '--help' ||
    command === 'help'
  ) {
    console.log(`
    Description
      This is the Tauri CLI.
    Usage
      $ tauri ${[...cmds, ...rustCliCmds].join('|')}
    Options
      --help, -h     Displays this message
      --version, -v  Displays the Tauri CLI version
    `)
    process.exit(0)
    // eslint-disable-next-line no-unreachable
    return false // do this for node consumers and tests
  }

  if (command === '-v' || command === '--version') {
    console.log(require('../package.json').version)
    return false // do this for node consumers and tests
  }

  if (cmds.includes(command)) {
    if (process.argv && !process.env.test) {
      process.argv.splice(2, 1)
    }
    console.log(`[tauri]: running ${command}`)
    // eslint-disable-next-line security/detect-non-literal-require
    require(`./tauri-${command}`)
  } else {
    console.log(`Invalid command ${command}. Use one of ${cmds.join(', ')}.`)
  }
}
module.exports = {
  tauri
}

tauri(cmd)
