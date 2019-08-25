const
  parseArgs = require('minimist'),
  path = require('path'),
  { writeFileSync } = require('fs-extra')

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

const appPaths = require('../helpers/app-paths'),
  Runner = require('../runner'),
  tauri = new Runner(appPaths),
  tauriConfig = require('../helpers/tauri-config')({
    ctx: {
      debug: true
    }
  })

const { bundle, ...cfg } = tauriConfig.tauri,
  cfgDir = injector.configDir()

require('../generator').generate(tauriConfig.tauri)
require('../entry').generate(appPaths.tauriDir, tauriConfig, true)

tauri.run(tauriConfig)
