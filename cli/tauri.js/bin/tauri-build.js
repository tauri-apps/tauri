const
  parseArgs = require('minimist')

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

const Runner = require('../runner')
const tauri = new Runner()
const tauriConfig = require('../helpers/tauri-config')({
  ctx: {
    debug: argv.debug,
    prod: true
  }
})

tauri.build(tauriConfig)
