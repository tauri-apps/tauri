const parseArgs = require('minimist')

const argv = parseArgs(process.argv.slice(2), {
  alias: {
    h: 'help',
    d: 'debug',
    t: 'target'
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
    --debug, -d    Builds with the debug flag
    --target, -t   Comma-separated list of target triples to build against
  `)
  process.exit(0)
}

const build = require('../dist/api/build')

build({
  ctx: {
    debug: argv.debug,
    target: argv.target
  }
})
