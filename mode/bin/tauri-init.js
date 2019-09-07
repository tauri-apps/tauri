const
    parseArgs = require('minimist'),
    appPaths = require('../helpers/app-paths'),
    logger = require('../helpers/logger'),
    log = logger('app:tauri'),
    warn = logger('app:tauri (init)', 'red')

const argv = parseArgs(process.argv.slice(2), {
    alias: {
        h: 'help'
    },
    boolean: ['h']
})

if (argv.help) {
    console.log(`
  Description
    Inits the Tauri template. If Tauri cannot find the tauri.conf.js
    it will create one.
  Usage
    $ tauri init
  Options
    --help, -h     Displays this message
  `)
    process.exit(0)
}

const { inject } = require('../template')

const target = appPaths.tauriDir

// if (!target) { // do it right here
if (inject(target, 'conf')) {
  log('tauri.conf.js successfully created')
} else {
  warn('tauri.conf.js exists, not overwriting')
}

if (inject(target, 'template')) {
  log('Tauri template successfully installed')
} else {
  warn('Tauri template exists, not overwriting')
}
