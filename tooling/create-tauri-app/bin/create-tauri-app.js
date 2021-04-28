#!/usr/bin/env node
// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const parseArgs = require('minimist')
const inquirer = require('inquirer')
const { resolve, join } = require('path')
const {
  recipeShortNames,
  recipeDescriptiveNames,
  recipeByDescriptiveName,
  recipeByShortName,
  install,
  checkPackageManager,
  shell,
  addTauriScript
} = require('../dist/')

/**
 * @type {object}
 * @property {boolean} h
 * @property {boolean} help
 * @property {boolean} v
 * @property {boolean} version
 * @property {string|boolean} f
 * @property {string|boolean} force
 * @property {boolean} l
 * @property {boolean} log
 * @property {boolean} d
 * @property {boolean} directory
 * @property {boolean} dev
 * @property {string} r
 * @property {string} recipe
 */
const createTauriApp = async (cliArgs) => {
  const argv = parseArgs(cliArgs, {
    alias: {
      h: 'help',
      v: 'version',
      f: 'force',
      l: 'log',
      m: 'manager',
      d: 'directory',
      dev: 'dev',
      t: 'tauri-path',
      A: 'app-name',
      W: 'window-title',
      D: 'dist-dir',
      P: 'dev-path',
      r: 'recipe'
    },
    boolean: ['h', 'l', 'ci', 'dev']
  })

  if (argv.help) {
    printUsage()
    return 0
  }

  if (argv.v) {
    console.log(require('../package.json').version)
    return false // do this for node consumers and tests
  }

  return getOptionsInteractive(argv, !argv.ci).then((responses) =>
    runInit(argv, responses)
  )
}

function printUsage() {
  console.log(`
  Description
    Starts a new tauri app from a "recipe" or pre-built template.
  Usage
    $ yarn create tauri-app <app-name> # npm create-tauri-app <app-name>
  Options
    --help, -h           Displays this message
    -v, --version        Displays the Tauri CLI version
    --ci                 Skip prompts
    --force, -f          Force init to overwrite [conf|template|all]
    --log, -l            Logging [boolean]
    --manager, -d        Set package manager to use [npm|yarn]
    --directory, -d      Set target directory for init
    --binary, -b         Optional path to a tauri binary from which to run init
    --app-name, -A       Name of your Tauri application
    --window-title, -W   Window title of your Tauri application
    --dist-dir, -D       Web assets location, relative to <project-dir>/src-tauri
    --dev-path, -P       Url of your dev server
    --recipe, -r         Add UI framework recipe. None by default.
                         Supported recipes: [${recipeShortNames.join('|')}]
    `)
}

const getOptionsInteractive = (argv, ask) => {
  const defaults = {
    appName: argv.A || 'tauri-app',
    tauri: { window: { title: 'Tauri App' } },
    recipeName: argv.r || 'vanillajs'
  }

  return inquirer
    .prompt([
      {
        type: 'input',
        name: 'appName',
        message: 'What is your app name?',
        default: defaults.appName,
        when: ask && !argv.A
      },
      {
        type: 'input',
        name: 'tauri.window.title',
        message: 'What should the window title be?',
        default: defaults.tauri.window.title,
        when: ask && !argv.W
      },
      {
        type: 'list',
        name: 'recipeName',
        message: 'Would you like to add a UI recipe?',
        choices: recipeDescriptiveNames,
        default: defaults.recipeName,
        when: ask && !argv.r
      }
    ])
    .then((answers) => ({
      ...defaults,
      ...answers
    }))
    .catch((error) => {
      if (error.isTtyError) {
        // Prompt couldn't be rendered in the current environment
        console.warn(
          'It appears your terminal does not support interactive prompts. Using default values.'
        )
        runInit()
      } else {
        // Something else went wrong
        console.error('An unknown error occurred:', error)
      }
    })
}

async function runInit(argv, config = {}) {
  const {
    appName,
    recipeName,
    tauri: {
      window: { title }
    }
  } = config
  // this little fun snippet pulled from vite determines the package manager the script was run from
  const packageManager = /yarn/.test(process.env.npm_execpath) ? 'yarn' : 'npm'

  let recipe

  if (argv.r) {
    recipe = recipeByShortName(argv.r)
  } else if (recipeName !== undefined) {
    recipe = recipeByDescriptiveName(recipeName)
  }

  let buildConfig = {
    distDir: argv.D,
    devPath: argv.P
  }

  if (recipe !== undefined) {
    buildConfig = recipe.configUpdate({ buildConfig, packageManager })
  }

  const directory = argv.d || process.cwd()
  const cfg = {
    ...buildConfig,
    appName: appName || argv.A,
    windowTitle: title || argv.w
  }

  // note that our app directory is reliant on the appName and
  // generally there are issues if the path has spaces (see Windows)
  // future TODO prevent app names with spaces or escape here?
  const appDirectory = join(directory, cfg.appName)

  // this throws an error if we can't run the package manager they requested
  await checkPackageManager({ cwd: directory, packageManager })

  if (recipe.preInit) {
    console.log('===== running initial command(s) =====')
    await recipe.preInit({ cwd: directory, cfg, packageManager })
  }

  const initArgs = [
    ['--app-name', cfg.appName],
    ['--window-title', cfg.windowTitle],
    ['--dist-dir', cfg.distDir],
    ['--dev-path', cfg.devPath]
  ].reduce((final, argSet) => {
    if (argSet[1]) {
      return final.concat(argSet)
    } else {
      return final
    }
  }, [])

  // Vue CLI plugin automatically runs these
  if (recipe.shortName !== 'vuecli') {
    console.log('===== installing any additional needed deps =====')
    if (argv.dev) {
      await shell('yarn', ['link', '@tauri-apps/cli'], {
        cwd: appDirectory
      })
      await shell('yarn', ['link', '@tauri-apps/api'], {
        cwd: appDirectory
      })
    }

    await install({
      appDir: appDirectory,
      dependencies: recipe.extraNpmDependencies,
      devDependencies: argv.dev
        ? [...recipe.extraNpmDevDependencies]
        : ['@tauri-apps/cli'].concat(recipe.extraNpmDevDependencies),
      packageManager
    })

    console.log('===== running tauri init =====')
    addTauriScript(appDirectory)

    const binary = !argv.b ? packageManager : resolve(appDirectory, argv.b)
    const runTauriArgs =
      packageManager === 'npm' && !argv.b
        ? ['run', 'tauri', '--', 'init']
        : ['tauri', 'init']
    await shell(binary, [...runTauriArgs, ...initArgs, '--ci'], {
      cwd: appDirectory
    })
  }

  if (recipe.postInit) {
    console.log('===== running final command(s) =====')
    await recipe.postInit({
      cwd: appDirectory,
      cfg,
      packageManager
    })
  }
}

createTauriApp(process.argv.slice(2)).catch((err) => {
  console.error(err)
})
