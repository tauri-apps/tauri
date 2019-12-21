const parseArgs = require('minimist')
const { appDir, tauriDir } = require('../helpers/app-paths')
const logger = require('../helpers/logger')
const log = logger('app:tauri')
const warn = logger('app:tauri (icon)', 'red')
const { tauricon } = require('../api/tauricon')
const { resolve } = require('path')

// TODO: Move tauricon.js to typescript

// /**
//  * @type {object}
//  * @property {boolean} h
//  * @property {boolean} help
//  * @property {string|boolean} f
//  * @property {string|boolean} force
//  * @property {boolean} l
//  * @property {boolean} log
//  * @property {boolean} c
//  * @property {boolean} config
//  * @property {boolean} s
//  * @property {boolean} source
//  * @property {boolean} t
//  * @property {boolean} target
//  */
// const argv = parseArgs(process.argv.slice(2), {
//   alias: {
//     h: 'help',
//     l: 'log',
//     c: 'config',
//     s: 'source',
//     t: 'target'
//   },
//   boolean: ['h', 'l']
// })

// if (argv.help) {
//   console.log(`
//   Description
//     Create all the icons you need for your Tauri app.

//   Usage
//     $ tauri icon

//   Options
//     --help, -h          Displays this message
//     --log, l            Logging [boolean]
//     --icon, i           Source icon (png, 1240x1240 with transparency)
//     --target, t         Target folder (default: 'src-tauri/icons')
//     --compression, c    Compression type [pngquant|optipng|zopfli]
//     `)
//   process.exit(0)
// }

// tauricon.make(
//   argv.i || resolve(appDir, 'app-icon.png'),
//   argv.t || resolve(tauriDir, 'icons'),
//   argv.c || 'optipng'
// ).then(() => {
//   log('(tauricon) Completed')
// }).catch(e => {
//   warn(e)
// })
