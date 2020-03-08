const parseArgs = require('minimist')

const argv = parseArgs(process.argv.slice(2), {
  alias: {
    h: 'help',
    e: 'exit-on-panic'
  },
  boolean: ['h', 'e']
})

if (argv.help) {
  console.log(`
  Description
    Tauri dev.
  Usage
    $ tauri dev
  Options
    --help, -h     Displays this message
  `)
  process.exit(0)
}

const dev = require('../dist/api/dev')

dev({
  ctx: {
    exitOnPanic: argv['exit-on-panic']
  }
})
