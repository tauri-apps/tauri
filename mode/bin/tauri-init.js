const
    parseArgs = require('minimist'),
    appPaths = require('../helpers/app-paths'),
    logger = require('../helpers/logger'),
    log = logger('app:tauri'),
    warn = logger('app:tauri (init)', 'red')

/**
 * @type {object}
 * @property {boolean} h
 * @property {boolean} help
 * @property {string|boolean} f
 * @property {string|boolean} force
 * @property {boolean} l
 * @property {boolean} log
 */
const argv = parseArgs(process.argv.slice(2), {
    alias: {
        h: 'help',
        f: 'force',
        l: 'log'
    },
    boolean: ['h', 'l']
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
    --force, -f    Force init to overwrite [conf|template|all]  
    --log, l       Logging [boolean]
  `)
    process.exit(0)
}

const { inject } = require('../template')

const target = appPaths.tauriDir

// if (!target) { // do it right here
if (inject(target, 'all', argv.f, argv.l)) {
  log('tauri init successful')
} else {
  warn('tauri init unsuccessful')
}
