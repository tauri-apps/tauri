#!/usr/bin/env node
// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const chalk = require('chalk')
const pkg = require('../package.json')
const updateNotifier = require('update-notifier')

const cmds = ['icon', 'deps']
const rustCliCmds = ['dev', 'build', 'init', 'info', 'sign']

const cmd = process.argv[2]
/**
 * @description This is the bootstrapper that in turn calls subsequent
 * Tauri Commands
 *
 * @param {string|array} command
 */
const tauri = async function (command) {
  // notifying updates.
  if (!(process.argv || []).some((arg) => arg === '--no-update-notifier')) {
    updateNotifier({
      pkg,
      updateCheckInterval: 0
    }).notify()
  }

  if (typeof command === 'object') {
    // technically we just care about an array
    command = command[0]
  }

  if (rustCliCmds.includes(command)) {
    const { runOnRustCli } = require('../dist/helpers/rust-cli')
    if (process.argv && !process.env.test) {
      process.argv.splice(0, 3)
    }
    ;(
      await runOnRustCli(
        command,
        (process.argv || []).filter((v) => v !== '--no-update-notifier')
      )
    ).promise.then(() => {
      if (command === 'init') {
        const {
          installDependencies
        } = require('../dist/api/dependency-manager')
        return installDependencies()
      }
    })
  } else {
    if (
      !command ||
      command === '-h' ||
      command === '--help' ||
      command === 'help'
    ) {
      console.log(`
        ${chalk.cyan(`
      :oooodddoooo;     ;oddl,      ,ol,       ,oc,  ,ldoooooooc,    ,oc,
      ';;;cxOx:;;;'    ;xOxxko'     :kx:       lkd,  :xkl;;;;:okx:   lkd,
          'dOo'       'oOd;:xkc     :kx:       lkd,  :xx:     ;xkc   lkd,
          'dOo'       ckx:  lkx;    :kx:       lkd,  :xx:     :xkc   lkd,
          'dOo'      ;xkl   ,dko'   :kx:       lkd,  :xx:.....xko,   lkd,
          'dOo'     'oOd,    :xkc   :kx:       lkd,  :xx:,;cokko'    lkd,
          'dOo'     ckk:      lkx;  :kx:       lkd,  :xx:    ckkc    lkd,
          'dOo'    ;xOl        lko; :xkl;,....;oOd,  :xx:     :xkl'  lkd,
          'okl'    'kd'        'xx'  'dxxxddddxxo'   :dd;      ;dxc  'xo'`)}

${chalk.yellow('Description')}
This is the Tauri CLI
${chalk.yellow('Usage')}
$ tauri ${[...rustCliCmds, ...cmds].join('|')}
${chalk.yellow('Options')}
--help, -h     Displays this message
--version, -v  Displays the Tauri CLI version
      `)

      process.exit(0)
      // eslint-disable-next-line no-unreachable
      return false // do this for node consumers and tests
    }

    if (command === '-v' || command === '--version') {
      console.log(`${pkg.version}`)
      return false // do this for node consumers and tests
    }

    if (cmds.includes(command)) {
      if (process.argv && !process.env.test) {
        process.argv.splice(2, 1)
      }
      console.log(`[tauri]: running ${command}`)
      require(`./tauri-${command}`)
    } else {
      console.log(`Invalid command ${command}. Use one of ${cmds.join(', ')}.`)
    }
  }
}

module.exports = {
  tauri
}

tauri(cmd).catch((e) => {
  throw e
})
