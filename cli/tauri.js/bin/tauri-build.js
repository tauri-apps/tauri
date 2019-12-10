const parseArgs = require('minimist')

const argv = parseArgs(process.argv.slice(2), {
  alias: {
    h: 'help',
    d: 'debug'
  },
  boolean: ['h', 'd']
})

if (argv.help) {
  console.log(`
  Description
    Tauri build.
  Usage
    $ tauri build
  Options
    --help, -h     Displays this message
  `)
  process.exit(0)
}

const build = require('../api/build')

build({ ctx: { debug: argv.debug } })
