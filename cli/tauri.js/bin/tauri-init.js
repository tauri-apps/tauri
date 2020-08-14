const parseArgs = require('minimist')
const inquirer = require('inquirer')
const { resolve } = require('path')
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
    A: 'app-name',
    W: 'window-title',
    D: 'dist-dir',
    P: 'dev-path'
  },
  boolean: ['h', 'l', 'ci']
})

if (argv.help) {
  console.log(`
  Description
    Inits the Tauri template. If Tauri cannot find the tauri.conf.json
    it will create one.
  Usage
    $ tauri init
  Options
    --help, -h           Displays this message
    --ci                 Skip prompts
    --force, -f          Force init to overwrite [conf|template|all]
    --log, -l            Logging [boolean]
    --directory, -d      Set target directory for init
    --tauri-path, -t     Path of the Tauri project to use (relative to the cwd)
    --app-name, -A       Name of your Tauri application
    --window-title, -W   Window title of your Tauri application
    --dist-dir, -D       Web assets location, relative to <project-dir>/src-tauri
    --dev-path, -P       Url of your dev server
    `)
  process.exit(0)
}

let appName = argv.A
if (!appName) {
  try {
    const packageJson = JSON.parse(
      readFileSync(resolve(process.cwd(), 'package.json')).toString()
    )
    appName = packageJson.displayName || packageJson.name
  } catch {}
}

if (argv.ci) {
  runInit()
} else {
  inquirer
    .prompt([
      {
        type: 'input',
        name: 'appName',
        message: 'What is your app name?',
        default: appName
      },
      {
        type: 'input',
        name: 'tauri.window.title',
        message: 'What should the window title be?',
        default: 'Tauri App',
        when: () => !argv.W
      },
      {
        type: 'input',
        name: 'build.distDir',
        message:
          'Where are your web assets (HTML/CSS/JS) located, relative to the "<current dir>/src-tauri" folder that will be created?',
        default: '../dist',
        when: () => !argv.D
      },
      {
        type: 'input',
        name: 'build.devPath',
        message: 'What is the url of your dev server?',
        default: 'http://localhost:4000',
        when: () => !argv.P
      }
    ])
    .then((answers) => {
      runInit(answers)
    })
    .catch((error) => {
      if (error.isTtyError) {
        // Prompt couldn't be rendered in the current environment
        console.log(
          'It appears your terminal does not support interactive prompts. Using default values.'
        )
        runInit()
      } else {
        // Something else when wrong
        console.error('An unknown error occurred:', error)
      }
    })
}

async function runInit(config = {}) {
  const { appName, ...configOptions } = config
  const init = require('../dist/api/init')

  const directory = argv.d || process.cwd()
  init({
    directory,
    force: argv.f || null,
    logging: argv.l || null,
    tauriPath: argv.t || null,
    appName: appName || argv.A || null,
    customConfig: merge(configOptions, {
      build: {
        distDir: argv.D,
        devPath: argv.P
      },
      tauri: {
        window: {
          title: argv.W
        }
      }
    })
  })

  const { installDependencies } = require('../dist/api/dependency-manager')
  await installDependencies()
}
