#!/usr/bin/env node

const cmds = ['init', 'dev', 'build', 'help']

const cmd = process.argv[2]

const tauri = function (command) {
  if (!command || command === '-h' || command === '--help' || command === 'help') {
    console.log(`
    Description
      This is the Tauri CLI.
    Usage
      $ tauri ${cmds.join('|')}
    Options
      --help, -h     Displays this message
    `)
    process.exit(0)
    return false// do this for node consumers and tests
  }
  if (cmds.includes(command)) {
    if (process.argv) {
      process.argv.splice(2, 1)
    }
    console.log(`[tauri]: running ${command}`)
    require(`./tauri-${command}`)
  } else {
    console.log(`Invalid command ${command}. Use one of ${cmds.join(',')}.`)
  }
}
module.exports = { tauri }

tauri(cmd)
