// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import minimist from 'minimist'
import inquirer, { Answers, QuestionCollection } from 'inquirer'
import { resolve, join } from 'path'

import { TauriBuildConfig } from './types/config'
import { reactjs, reactts } from './recipes/react'
import { vuecli } from './recipes/vue-cli'
import { vanillajs } from './recipes/vanilla'
import { vite } from './recipes/vite'
import {
  install,
  checkPackageManager,
  PackageManager
} from './dependency-manager'

import { shell } from './shell'
import { addTauriScript } from './helpers/add-tauri-script'

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
  const argv = (minimist(cliArgs, {
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
  }) as unknown) as Argv

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

export interface RecipeArgs {
  cwd: string
  cfg: TauriBuildConfig
  packageManager: PackageManager
  ci: boolean
  // eslint-disable-next-line @typescript-eslint/no-invalid-void-type
  answers?: undefined | void | Answers
}
export interface Recipe {
  descriptiveName: string
  shortName: string
  configUpdate?: (args: RecipeArgs) => TauriBuildConfig
  extraNpmDependencies: string[]
  extraNpmDevDependencies: string[]
  extraQuestions?: (args: RecipeArgs) => QuestionCollection[]
  preInit?: (args: RecipeArgs) => Promise<void>
  postInit?: (args: RecipeArgs) => Promise<void>
}

const allRecipes: Recipe[] = [vanillajs, reactjs, reactts, vite, vuecli]

const recipeByShortName = (name: string): Recipe | undefined =>
  allRecipes.find((r) => r.shortName === name)

const recipeByDescriptiveName = (name: string): Recipe | undefined =>
  allRecipes.find((r) => r.descriptiveName === name)

const recipeShortNames = allRecipes.map((r) => r.shortName)

const recipeDescriptiveNames = allRecipes.map((r) => r.descriptiveName)

const runInit = async (argv: Argv): Promise<void> => {
  const defaults = {
    appName: 'tauri-app',
    tauri: { window: { title: 'Tauri App' } },
    recipeName: 'vanillajs'
  }

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

  if (!recipe) throw new Error('Could not find the recipe specified.')

  // this little fun snippet pulled from vite determines the package manager the script was run from
  const packageManager =
    argv.m === 'yarn' || argv.m === 'npm'
      ? argv.m
      : // @ts-expect-error
      /yarn/.test(process?.env?.npm_execpath)
      ? 'yarn'
      : 'npm'

  const buildConfig = {
    distDir: argv.D,
    devPath: argv.P,
    appName: appName,
    windowTitle: title
  }

  const directory = argv.d || process.cwd()

  // prompt recipe questions
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
    console.log('===== running initial command(s) =====')
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
      packageManager,
      ci: argv.ci,
      answers: recipeAnswers
    })
  }
}
