const parseArgs = require('minimist')
const inquirer = require('inquirer')
const { resolve } = require('path')
const { merge } = require('lodash')

/**
 * init creates the src-tauri folder
 *
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
function main(cliArgs) {
  const argv = parseArgs(cliArgs, {
    alias: {
      h: 'help',
      f: 'force',
      l: 'log',
      A: 'app-name',
      W: 'window-title',
      D: 'dist-dir',
      P: 'dev-path'
    },
    boolean: ['h', 'l', 'ci']
  })

  if (argv.help) {
    printUsage()
    process.exit(0)
  }

  if (argv.ci) {
    runInit(argv)
  } else {
    getOptionsInteractive(argv).then((responses) => runInit(argv, responses))
  }
}

function printUsage() {
  console.log(`
  Description
    Inits the Tauri template. If Tauri cannot find the tauri.conf.json
    it will create one.
  Usage
    $ tauri init
  Options
    --help, -h           Displays this message
    --app-name, -a       Name of your Tauri application
    --window-title, -w   Window title of your Tauri application
    --dist-dir, -d       Web assets location, relative to <project-dir>/src-tauri
    --dev-path, -p       Url of your dev server
    --ci                 Skip prompts
    --force, -f          Force init to overwrite [conf|template|all]
    --log, -l            Logging [boolean]
    --tauri-path, -t     Path of the Tauri project to use (relative to the cwd)
    --directory, -D      Set target directory for init
    `)
}

const getOptionsInteractive = (argv) => {
  let defaultAppName = argv.a
  if (!defaultAppName) {
    try {
      const packageJson = JSON.parse(
        readFileSync(resolve(process.cwd(), 'package.json')).toString()
      )
      defaultAppName = packageJson.displayName || packageJson.name
    } catch {}
  }

  return inquirer
    .prompt([
      {
        type: 'input',
        name: 'appName',
        message: 'What is your app name?',
        default: defaultAppName,
        when: !argv.a
      },
      {
        type: 'input',
        name: 'windowTitle',
        message: 'What should the window title be?',
        default: 'Tauri App',
        when: () => !argv.w
      },
      {
        type: 'input',
        name: 'build.devPath',
        message: 'What is the url of your dev server?',
        default: 'http://localhost:4000',
        when: () => !argv.p
      },
      {
        type: 'input',
        name: 'build.distDir',
        message:
          'Where are your web assets (HTML/CSS/JS) located, relative to the "<current dir>/src-tauri" folder that will be created?',
        default: '../dist',
        when: () => !argv.d
      }
    ])
    .then((answers) => answers)
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

async function runInit(argv, config = {}) {
  const { appName, windowTitle, ...configOptions } = config
  const init = require('../dist/api/init')

  let buildConfig = {
    distDir: argv.d,
    devPath: argv.p
  }

  const directory = argv.D || process.cwd()

  init({
    directory,
    force: argv.f || null,
    logging: argv.l || null,
    tauriPath: argv.t || null,
    appName: appName || argv.a || null,
    customConfig: merge(configOptions, {
      build: buildConfig,
      tauri: {
        windows: [
          {
            title: windowTitle || argv.w
          }
        ]
      }
    })
  })

  const { installDependencies } = require('../dist/api/dependency-manager')
  await installDependencies()
}

module.exports = main
