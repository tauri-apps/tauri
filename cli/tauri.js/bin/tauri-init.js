const parseArgs = require('minimist')

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
    d: 'directory',
    t: 'tauri-path'
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
    --help, -h        Displays this message
    --force, -f       Force init to overwrite [conf|template|all]
    --log, -l         Logging [boolean]
    --directory, -d   Set target directory for init
    --tauri-path, -t   Path of the Tauri project to use (relative to the cwd)
    `)
  process.exit(0)
}

const init = require('../dist/init')

init({
  directory: argv.d || process.cwd(),
  force: argv.f || null,
  logging: argv.l || null,
  tauriPath: argv.t || null
})
