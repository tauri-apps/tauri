const parseArgs = require('minimist')
const { tauricon } = require('../dist/api/tauricon')

/**
 * @type {object}
 * @property {boolean} h
 * @property {boolean} help
 * @property {string|boolean} f
 * @property {string|boolean} force
 * @property {boolean} l
 * @property {boolean} log
 * @property {boolean} c
 * @property {boolean} config
 * @property {boolean} s
 * @property {boolean} source
 * @property {boolean} t
 * @property {boolean} target
 */
const argv = parseArgs(process.argv.slice(2), {
  alias: {
    h: 'help',
    l: 'log',
    c: 'config',
    s: 'source',
    t: 'target'
  },
  boolean: ['h', 'l']
})

if (argv.help) {
  console.log(`
  Description
    Create all the icons you need for your Tauri app.

  Usage
    $ tauri icon

  Options
    --help, -h          Displays this message
    --log, l            Logging [boolean]
    --icon, i           Source icon (png, 1240x1240 with transparency)
    --target, t         Target folder (default: 'src-tauri/icons')
    --compression, c    Compression type [pngquant|optipng|zopfli]
    `)
  process.exit(0)
}

tauricon.make(
  argv.i,
  argv.t,
  argv.c || 'optipng'
).then(() => {
  // TODO: use logger module for prettier output
  console.log('app:tauri (tauricon) Completed')
}).catch(e => {
  // TODO: use logger module for prettier output
  console.error('app:tauri (icon)', e)
})
