#!/usr/bin/env node

const cmds = ['create', 'init', 'dev', 'build', 'help', 'icon', 'info', 'deps']

const cmd = process.argv[2]
/**
 * @description This is the bootstrapper that in turn calls subsequent
 * Tauri Commands
 *
 * @param {string|array} command
 */
const tauri = function (command) {
  if (typeof command === 'object') { // technically we just care about an array
    command = command[0]
  }

  if (!command || command === '-h' || command === '--help' || command === 'help') {
    console.log(`
    Description
      This is the Tauri CLI.
    Usage
      $ tauri ${cmds.join('|')}
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
    if (['create', 'init'].includes(command)) {
      require(`./tauri-${command}`)(process.argv.slice(2))
    } else {
      require(`./tauri-${command}`)
    }
  } else {
    console.log(`Invalid command ${command}. Use one of ${cmds.join(',')}.`)
  }
}
module.exports = {
  tauri
}

tauri(cmd)
