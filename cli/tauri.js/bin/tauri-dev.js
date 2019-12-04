const parseArgs = require('minimist')

const argv = parseArgs(process.argv.slice(2), {
  alias: {
    h: 'help'
  },
  boolean: ['h']
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

const Runner = require('../runner')
const tauri = new Runner()
const tauriConfig = require('../helpers/tauri-config')({
  ctx: {
    debug: true
  }
})

tauri.run(tauriConfig)
