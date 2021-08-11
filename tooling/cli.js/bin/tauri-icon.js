// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import parseArgs from 'minimist'
import tauricon from '../dist/api/tauricon.js'

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
    t: 'target'
  },
  boolean: ['h', 'l']
})

if (argv.help) {
  console.log(`
  Description
    Create all the icons you need for your Tauri app.
    The icon path is the source icon (png, 1240x1240 with transparency).

  Usage
    $ tauri icon [ICON-PATH]

  Options
    --help, -h          Displays this message
    --log, l            Logging [boolean]
    --target, t         Target folder (default: 'src-tauri/icons')
    --compression, c    Compression type [optipng|zopfli]
    --ci                Runs the script in CI mode     
    `)
  process.exit(0)
}

tauricon
  .make(argv._[0], argv.t, argv.c || 'optipng')
  .then(() => {
    // TODO: use logger module for prettier output
    console.log('app:tauri (tauricon) Completed')
  })
  .catch((e) => {
    // TODO: use logger module for prettier output
    console.error('app:tauri (icon)', e)
  })
