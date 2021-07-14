// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import minimist from 'minimist'
import inquirer from 'inquirer'
import { bold, cyan, green, reset, yellow } from 'chalk'
import { platform } from 'os'
import { resolve, join, relative } from 'path'
import { cra } from './recipes/react'
import { vuecli } from './recipes/vue-cli'
import { vanillajs } from './recipes/vanilla'
import { vite } from './recipes/vite'
import { ngcli } from './recipes/ng-cli'
import { install, checkPackageManager } from './dependency-manager'
import { shell } from './shell'
import { addTauriScript } from './helpers/add-tauri-script'
import { Recipe } from './types/recipe'
import { updateTauriConf } from './helpers/update-tauri-conf'

interface Argv {
  h: boolean
  help: boolean
  v: string
  version: string
  ci: boolean
  dev: boolean
  b: string
  binary: string
  f: string
  force: string
  l: boolean
  log: boolean
  m: string
  manager: string
  d: string
  directory: string
  t: string
  tauriPath: string
  A: string
  appName: string
  W: string
  windowTitle: string
  D: string
  distDir: string
  P: string
  devPath: string
  r: string
  recipe: string
}

const printUsage = (): void => {
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
    --app-name, -A       Name of your Tauri application
    --window-title, -W   Window title of your Tauri application
    --dist-dir, -D       Web assets location, relative to <project-dir>/src-tauri
    --dev-path, -P       Url of your dev server
    --recipe, -r         Add UI framework recipe. None by default.
                         Supported recipes: [${recipeShortNames.join('|')}]
    `)
}

export const createTauriApp = async (cliArgs: string[]): Promise<any> => {
  const argv = minimist(cliArgs, {
    alias: {
      h: 'help',
      v: 'version',
      f: 'force',
      l: 'log',
      m: 'manager',
      d: 'directory',
      t: 'tauri-path',
      A: 'app-name',
      W: 'window-title',
      D: 'dist-dir',
      P: 'dev-path',
      r: 'recipe'
    },
    boolean: ['h', 'l', 'ci', 'dev']
  }) as unknown as Argv

  if (argv.help) {
    printUsage()
    return 0
  }

  if (argv.v) {
    /* eslint-disable @typescript-eslint/no-var-requires */
    /* eslint-disable @typescript-eslint/no-unsafe-member-access */
    console.log(require('../package.json').version)
    return false // do this for node consumers and tests
    /* eslint-enable @typescript-eslint/no-var-requires */
    /* eslint-enable @typescript-eslint/no-unsafe-member-access */
  }

  return await runInit(argv)
}

interface Responses {
  appName: string
  tauri: { window: { title: string } }
  recipeName: string
}

const allRecipes: Recipe[] = [vanillajs, cra, vite, vuecli, ngcli]

const recipeByShortName = (name: string): Recipe | undefined =>
  allRecipes.find((r) => r.shortName === name)

const recipeByDescriptiveName = (name: string): Recipe | undefined =>
  allRecipes.find((r) => r.descriptiveName.value === name)

const recipeShortNames = allRecipes.map((r) => r.shortName)

const recipeDescriptiveNames = allRecipes.map((r) => r.descriptiveName)

const keypress = async (skip: boolean): Promise<void> => {
  if (skip) return
  process.stdin.setRawMode(true)
  return await new Promise((resolve, reject) => {
    console.log('Press any key to continue...')
    process.stdin.once('data', (data) => {
      const byteArray = [...data]
      if (byteArray.length > 0 && byteArray[0] === 3) {
        console.log('^C')
        process.exit(1)
      }
      process.stdin.setRawMode(false)
      resolve()
    })
  })
}

const runInit = async (argv: Argv): Promise<void> => {
  console.log(
    `We hope to help you create something special with ${bold(
      yellow('Tauri')
    )}!`
  )
  console.log(
    'You will have a choice of one of the UI frameworks supported by the greater web tech community.'
  )
  console.log(
    `This should get you started. See our docs at https://tauri.studio/`
  )

  const setupLink =
    platform() === 'win32'
      ? 'https://tauri.studio/en/docs/getting-started/setup-windows/'
      : platform() === 'darwin'
      ? 'https://tauri.studio/en/docs/getting-started/setup-macos/'
      : 'https://tauri.studio/en/docs/getting-started/setup-linux/'

  console.log(
    `If you haven't already, please take a moment to setup your system.`
  )
  console.log(`You may find the requirements here: ${setupLink}`)

  await keypress(argv.ci)

  const defaults = {
    appName: 'tauri-app',
    tauri: { window: { title: 'Tauri App' } },
    recipeName: 'vanillajs'
  }

  // prompt initial questions
  const answers = (await inquirer
    .prompt([
      {
        type: 'input',
        name: 'appName',
        message: 'What is your app name?',
        default: defaults.appName,
        when: !argv.ci && !argv.A
      },
      {
        type: 'input',
        name: 'tauri.window.title',
        message: 'What should the window title be?',
        default: defaults.tauri.window.title,
        when: !argv.ci && !argv.W
      },
      {
        type: 'list',
        name: 'recipeName',
        message: 'Would you like to add a UI recipe?',
        choices: recipeDescriptiveNames,
        default: defaults.recipeName,
        when: !argv.ci && !argv.r
      }
    ])
    .catch((error: { isTtyError: boolean }) => {
      if (error.isTtyError) {
        // Prompt couldn't be rendered in the current environment
        console.warn(
          'It appears your terminal does not support interactive prompts. Using default values.'
        )
      } else {
        // Something else went wrong
        console.error('An unknown error occurred:', error)
      }
    })) as Responses

  const {
    appName,
    recipeName,
    tauri: {
      window: { title }
    }
  } = { ...defaults, ...answers }

  let recipe: Recipe | undefined
  if (argv.r) {
    recipe = recipeByShortName(argv.r)
  } else if (recipeName !== undefined) {
    recipe = recipeByDescriptiveName(recipeName)
  }

  // throw if recipe is not set
  if (!recipe) {
    throw new Error('Could not find the recipe specified.')
  }

  const packageManager =
    argv.m === 'yarn' || argv.m === 'npm'
      ? argv.m
      : // @ts-expect-error
      // this little fun snippet pulled from vite determines the package manager the script was run from
      /yarn/.test(process?.env?.npm_execpath)
      ? 'yarn'
      : 'npm'

  const buildConfig = {
    distDir: argv.D,
    devPath: argv.P,
    appName: argv.A || appName,
    windowTitle: argv.W || title
  }

  const directory = argv.d || process.cwd()

  // prompt additional recipe questions
  let recipeAnswers
  if (recipe.extraQuestions) {
    recipeAnswers = await inquirer
      .prompt(
        recipe.extraQuestions({
          cfg: buildConfig,
          packageManager,
          ci: argv.ci,
          cwd: directory
        })
      )
      .catch((error: { isTtyError: boolean }) => {
        if (error.isTtyError) {
          // Prompt couldn't be rendered in the current environment
          console.warn(
            'It appears your terminal does not support interactive prompts. Using default values.'
          )
        } else {
          // Something else went wrong
          console.error('An unknown error occurred:', error)
        }
      })
  }

  let updatedConfig
  if (recipe.configUpdate) {
    updatedConfig = recipe.configUpdate({
      cfg: buildConfig,
      packageManager,
      ci: argv.ci,
      cwd: directory,
      answers: recipeAnswers
    })
  }
  const cfg = {
    ...buildConfig,
    ...(updatedConfig ?? {})
  }

  // note that our app directory is reliant on the appName and
  // generally there are issues if the path has spaces (see Windows)
  // future TODO prevent app names with spaces or escape here?
  const appDirectory = join(directory, cfg.appName)

  // this throws an error if we can't run the package manager they requested
  await checkPackageManager({ cwd: directory, packageManager })

  if (recipe.preInit) {
    logStep('Running initial command(s)')
    await recipe.preInit({
      cwd: directory,
      cfg,
      packageManager,
      ci: argv.ci,
      answers: recipeAnswers
    })
  }

  const initArgs = [
    ['--app-name', cfg.appName],
    ['--window-title', cfg.windowTitle],
    ['--dist-dir', cfg.distDir],
    ['--dev-path', cfg.devPath]
  ].reduce((final: string[], argSet) => {
    if (argSet[1]) {
      return final.concat(argSet)
    } else {
      return final
    }
  }, [])

  const tauriCLIVersion = !argv.dev
    ? 'latest'
    : `file:${relative(appDirectory, join(__dirname, '../../cli.js'))}`

  // Vue CLI plugin automatically runs these
  if (recipe.shortName !== 'vuecli') {
    logStep('Installing any additional needed dependencies')
    await install({
      appDir: appDirectory,
      dependencies: recipe.extraNpmDependencies,
      devDependencies: [`@tauri-apps/cli@${tauriCLIVersion}`].concat(
        recipe.extraNpmDevDependencies
      ),
      packageManager
    })

    logStep(`Adding ${reset(yellow('"tauri"'))} script to package.json`)
    addTauriScript(appDirectory)

    logStep(`Running: ${reset(yellow('tauri init'))}`)
    const binary = !argv.b ? packageManager : resolve(appDirectory, argv.b)
    const runTauriArgs =
      packageManager === 'npm' && !argv.b
        ? ['run', 'tauri', '--', 'init']
        : ['tauri', 'init']
    await shell(binary, [...runTauriArgs, ...initArgs, '--ci'], {
      cwd: appDirectory
    })

    logStep(`Updating ${reset(yellow('"tauri.conf.json"'))}`)
    updateTauriConf(appDirectory, cfg)
  }

  if (recipe.postInit) {
    logStep('Running final command(s)')
    await recipe.postInit({
      cwd: appDirectory,
      cfg,
      packageManager,
      ci: argv.ci,
      answers: recipeAnswers
    })
  }
}

function logStep(msg: string): void {
  const out = `${green('>>')} ${bold(cyan(msg))}`
  console.log(out)
}
