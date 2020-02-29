const parseArgs = require('minimist')
const inquirer = require('inquirer')
const { merge } = require('lodash')

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
    t: 'tauri-path',
    w: 'window-title',
    D: 'distDir',
    p: 'devPath'
  },
  boolean: ['h', 'l']
})

if (argv.help) {
  console.log(`
  Description
    Inits the Tauri template. If Tauri cannot find the tauri.conf.json
    it will create one.
  Usage
    $ tauri init
  Options
    --help, -h          Displays this message
    --force, -f         Force init to overwrite [conf|template|all]
    --log, -l           Logging [boolean]
    --directory, -d     Set target directory for init
    --tauri-path, -t    Path of the Tauri project to use (relative to the cwd)
    --window-title, -w  Window title of your Tauri application
    --distDir, -D       Output directory of your web app build process
    --devPath, -p       Url of your dev server
    `)
  process.exit(0)
}

inquirer
  .prompt([
    {
      type: 'input',
      name: 'tauri.window.title',
      message: 'What should the window title be?',
      default: 'Tauri App',
      when: () => !argv.w
    },
    {
      type: 'input',
      name: 'build.distDir',
      message:
        'What is the output directory of your web app build process, relative to the "src-tauri" folder that will be created?',
      default: '../dist',
      when: () => !argv.D
    },
    {
      type: 'input',
      name: 'build.devPath',
      message: 'What is the url of your dev server?',
      default: 'http://localhost:4000',
      when: () => !argv.p
    }
  ])
  .then(answers => {
    runInit(answers)
  })
  .catch(error => {
    if (error.isTtyError) {
      // Prompt couldn't be rendered in the current environment
      console.log(
        'It appears your terminal does not support interactive prompts. Using default values.'
      )
      runInit()
    } else {
      // Something else when wrong
      console.error('An unknown error occurred')
    }
  })

function runInit(config = {}) {
  const init = require('../dist/api/init')

  init({
    directory: argv.d || process.cwd(),
    force: argv.f || null,
    logging: argv.l || null,
    tauriPath: argv.t || null,
    customConfig: merge(config, {
      build: {
        distDir: argv.D,
        devPath: argv.p
      },
      tauri: {
        window: {
          title: argv.w
        }
      }
    })
  })
}
