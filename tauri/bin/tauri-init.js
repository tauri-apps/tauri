const
    parseArgs = require('minimist'),
    appPaths = require('../helpers/app-paths'),
    log = require('../helpers/logger')('app:tauri')

const argv = parseArgs(process.argv.slice(2), {
    alias: {
        h: 'help'
    },
    boolean: ['h']
})

if (argv.help) {
    console.log(`
  Description
    Inits the Tauri template.
  Usage
    $ tauri init
  Options
    --help, -h     Displays this message
  `)
    process.exit(0)
}

const Injector = require('../injector'),
  injector = new Injector(appPaths)
injector.injectTemplate()
log('Tauri template successfully installed')
