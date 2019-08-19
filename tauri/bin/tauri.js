#!/usr/bin/env node

const cmds = ['init', 'dev', 'build', 'help']

const cmd = process.argv[2]
if (!cmd || cmd === '-h' || cmd === '--help' || cmd === 'help') {
  console.log(`
  Description
    Tauri CLI.
  Usage
    $ tauri ${cmds.join('|')}
  Options
    --help, -h     Displays this message
  `)
  process.exit(0)
}
if (cmds.includes(cmd)) {
   process.argv.splice(2, 1)
   require(`./tauri-${cmd}`)
} else {
  console.log(`Invalid command ${cmd}. Use one of ${cmds.join(',')}.`)
}
