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

const { tauriDir } = require('../helpers/app-paths'),
  Runner = require('../runner'),
  tauri = new Runner({modeDir: tauriDir}),
  tauriConfig = require('../helpers/tauri-config')({
    ctx: {
      debug: argv.debug
    }
  })

require('../generator').generate(tauriConfig.tauri)
require('../entry').generate(tauriDir, tauriConfig)

tauri.build(tauriConfig)
