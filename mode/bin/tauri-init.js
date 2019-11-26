const parseArgs = require('minimist')
const appPaths = require('../helpers/app-paths')
const logger = require('../helpers/logger')
const log = logger('app:tauri')
const warn = logger('app:tauri (init)', 'red')

/**
 * @type {object}
 * @property {boolean} h
 * @property {boolean} help
 * @property {string|boolean} f
 * @property {string|boolean} force
 * @property {boolean} l
 * @property {boolean} log
 * @property {boolean} d
 * @property {boolean} directory
 */
const argv = parseArgs(process.argv.slice(2), {
  alias: {
    h: 'help',
    f: 'force',
    l: 'log',
    d: 'directory'
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
    --help, -h       Displays this message
    --force, -f      Force init to overwrite [conf|template|all]
    --log, l         Logging [boolean]
    --directory, d   Set target directory for init
    `)
  process.exit(0)
}

const { inject } = require('../template')

const target = appPaths.tauriDir

if (inject(target, 'all', argv.f || null, argv.l || null, argv.d || null)) {
  log('tauri init successful')
} else {
  warn('tauri init unsuccessful')
}
