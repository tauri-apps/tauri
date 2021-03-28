#!/usr/bin/env node

const cmds = ['help', 'icon', 'deps']
const rustCliCmds = ['dev', 'build', 'init', 'info']

const cmd = process.argv[2]
const chalk = require('chalk')
const pkg = require('../package.json')
const figlet = require('figlet')
const updateNotifier = require('update-notifier')
/**
 * @description This is the bootstrapper that in turn calls subsequent
 * Tauri Commands
 *
 * @param {string|array} command
 */
let skipUpdateNotifier = false
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
    runOnRustCli(
      command,
      (process.argv || []).filter((v) => v !== '--no-update-notifier')
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
      console.log(chalk.cyan(figlet.textSync('Tauri')))
      console.log(
        `${chalk.cyan(
          'Description'
        )} \n This is the Tauri CLI \n ${chalk.magenta('Usage')} \n $ tauri ${[
          ...rustCliCmds,
          ...cmds
        ].join('|')} \n ${chalk.cyan(
          'Options'
        )} \n --help, -h     Displays this message \n  --version, -v  Displays the Tauri CLI version`
      )

      process.exit(0)
      // eslint-disable-next-line no-unreachable
      return false // do this for node consumers and tests
    }

    if (command === '-v' || command === '--version') {
      console.log(`${pkg.version}`)
      return false // do this for node consumers and tests
    }

    if ((process.argv || []).some((arg) => arg === '--no-update-notifier')) {
      skipUpdateNotifier = true
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

  // notifying updates.
  if (!skipUpdateNotifier) {
    updateNotifier({ pkg }).notify()
  }
}

module.exports = {
  tauri
}

tauri(cmd)
