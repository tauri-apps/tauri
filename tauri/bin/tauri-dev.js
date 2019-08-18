const
  parseArgs = require('minimist'),
  path = require('path')

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
  Injector = require('../injector'),
  tauri = new Runner(appPaths),
  injector = new Injector(appPaths),
  tauriConfig = require('../helpers/tauri-config')({
    ctx: {
      debug: true
    }
  })

const { bundle, ...cfg } = tauriConfig.tauri,
  cfgDir = injector.configDir()
console.log(JSON.stringify(cfg))
fse.writeFileSync(path.join(cfgDir, 'config.json'), JSON.stringify(cfg))
fse.writeFileSync(path.join(cfgDir, 'bundle.json'), JSON.stringify(bundle))

tauri.run(tauriConfig)
