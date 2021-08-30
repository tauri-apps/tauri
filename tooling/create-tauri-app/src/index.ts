// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import inquirer from 'inquirer'
import { program, createOption } from 'commander'
import { bold, cyan, green, reset, yellow } from 'chalk'
import { platform } from 'os'
import { resolve, join, relative } from 'path'
import { cra } from './recipes/react'
import { vuecli } from './recipes/vue-cli'
import { vanillajs } from './recipes/vanilla'
import { vite } from './recipes/vite'
import { dominator } from './recipes/dominator'
import { ngcli } from './recipes/ng-cli'
import { svelte } from './recipes/svelte'
import { install, checkPackageManager } from './dependency-manager'
import { shell } from './shell'
import { updatePackageJson } from './helpers/update-package-json'
import { Recipe } from './types/recipe'
import { updateTauriConf } from './helpers/update-tauri-conf'

const allRecipes: Recipe[] = [
  vanillajs,
  cra,
  vite,
  vuecli,
  ngcli,
  svelte,
  dominator
]
const recipeShortNames = allRecipes.map((r) => r.shortName)
const recipeDescriptiveNames = allRecipes.map((r) => r.descriptiveName)
const recipeByShortName = (name: string): Recipe | undefined =>
  allRecipes.find((r) => r.shortName === name)
const recipeByDescriptiveName = (name: string): Recipe | undefined =>
  allRecipes.find((r) => r.descriptiveName.value === name)

interface Argv {
  help: boolean
  version: string
  ci: boolean
  dev: boolean
  binary: string
  force: string
  log: boolean
  manager: string
  directory: string
  appName: string
  windowTitle: string
  distDir: string
  devPath: string
  recipe: string
}

export const createTauriApp = async (cliArgs: string[]): Promise<any> => {
  program
    .description(
      'Bootstrap a new tauri app from a "recipe" or pre-built template.'
    )
    .addOption(
      createOption(
        '-r, --recipe <recipe>',
        'Specify UI framework recipe'
      ).choices(recipeShortNames)
    )
    .option('  , --ci', 'Skip prompts')
    .option('  , --dev', 'Use local development packages')
    .addOption(
      createOption('-f, --force [option]', 'Force init to overwrite')
        .choices(['conf', 'template', 'all'])
        .default('all')
    )
    .option('-d, --directory <path>', 'Set target directory for init')
    .option('-A, --app-name <name>', 'Name of your Tauri application')
    .option(
      '-W, --window-title <title>',
      'Title of your Tauri application window'
    )
    .option(
      '-D, --dist-dir <path>',
      'Web assets location, relative to "<project-dir>/src-tauri/tauri.conf.json"'
    )
    .option('-p, --dev-path <path>', 'Url of your dev server')
    .addOption(
      createOption(
        '-m, --manager <package-manager>',
        'Set package manager to use'
      ).choices(['npm', 'yarn', 'pnpm'])
    )
    .addOption(
      createOption('-b, --binary <path>', 'Use a prebuilt Tauri CLI binary')
    )
    .option('-l, --log', 'Add log messages')
    .version(
      // eslint-disable-next-line
      require('../package.json').version,
      '-v, --version',
      'Displays create-tauri-app version'
    )
    .helpOption('-h, --help', 'Displays this message')
    .showHelpAfterError('For more information try --help')
    .configureHelp({
      optionTerm: (option) => cyan(option.flags),
      commandUsage: (command) => cyan(command.name()) + ' [options]',
      commandDescription: (command) => yellow(command.description())
    })
    .parse(process.argv)

  const argv = program.opts()
  return await runInit(argv as Argv)
}

interface Responses {
  appName: string
  tauri: { window: { title: string } }
  recipeName: string
  installApi: boolean
}

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
  const setupLink =
    platform() === 'win32'
      ? 'https://tauri.studio/en/docs/getting-started/setup-windows/'
      : platform() === 'darwin'
      ? 'https://tauri.studio/en/docs/getting-started/setup-macos/'
      : 'https://tauri.studio/en/docs/getting-started/setup-linux/'

  // prettier-ignore
  console.log(
    `
We hope to help you create something special with ${bold(yellow('Tauri'))}!
You will have a choice of one of the UI frameworks supported by the greater web tech community.
This tool should get you quickly started. See our docs at ${cyan('https://tauri.studio/')}

If you haven't already, please take a moment to setup your system.
You may find the requirements here: ${cyan(setupLink)}
    `
  )

  await keypress(argv.ci)

  const defaults = {
    appName: 'tauri-app',
    tauri: { window: { title: 'Tauri App' } },
    recipeName: 'Vanilla.js',
    installApi: true
  }

  // prompt initial questions
  const answers = (await inquirer
    .prompt([
      {
        type: 'input',
        name: 'appName',
        message: 'What is your app name?',
        default: defaults.appName,
        when: !argv.ci && !argv.appName
      },
      {
        type: 'input',
        name: 'tauri.window.title',
        message: 'What should the window title be?',
        default: defaults.tauri.window.title,
        when: !argv.ci && !argv.windowTitle
      },
      {
        type: 'list',
        name: 'recipeName',
        message: 'What UI recipe would you like to add?',
        choices: recipeDescriptiveNames,
        default: defaults.recipeName,
        when: !argv.ci && !argv.recipe
      },
      {
        type: 'confirm',
        name: 'installApi',
        message: 'Add "@tauri-apps/api" npm package?',
        default: true,
        when: !argv.ci
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
    installApi,
    tauri: {
      window: { title }
    }
  } = { ...defaults, ...answers }

  let recipe: Recipe | undefined
  if (argv.recipe) {
    recipe = recipeByShortName(argv.recipe)
  } else if (recipeName !== undefined) {
    recipe = recipeByDescriptiveName(recipeName)
  }

  // throw if recipe is not set
  if (!recipe) {
    throw new Error('Could not find the recipe specified.')
  }

  const packageManager =
    argv.manager === 'yarn' || argv.manager === 'npm' || argv.manager === 'pnpm'
      ? argv.manager
      : // @ts-expect-error
      // this little fun snippet pulled from vite determines the package manager the script was run from
      /yarn/.test(process?.env?.npm_execpath)
      ? 'yarn'
      : // @ts-expect-error
      /pnpm/.test(process?.env?.npm_execpath)
      ? 'pnpm'
      : 'npm'

  const buildConfig = {
    distDir: argv.distDir,
    devPath: argv.devPath,
    appName: argv.appName || appName,
    windowTitle: argv.windowTitle || title
  }

  const directory = argv.directory || process.cwd()

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
      answers: recipeAnswers ?? {}
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
      answers: recipeAnswers ?? {}
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
      dependencies: [installApi ? '@tauri-apps/api@latest' : ''].concat(
        recipe.extraNpmDependencies
      ),
      devDependencies: [`@tauri-apps/cli@${tauriCLIVersion}`].concat(
        recipe.extraNpmDevDependencies
      ),
      packageManager
    })

    logStep(`Updating ${reset(yellow('"package.json"'))}`)
    updatePackageJson(appDirectory, appName)

    logStep(`Running ${reset(yellow('"tauri init"'))}`)
    const binary = !argv.binary
      ? packageManager
      : resolve(appDirectory, argv.binary)
    // pnpm is equivalent to yarn and can run srcipts without using "run" but due to this bug https://github.com/pnpm/pnpm/issues/2764
    // we need to pass "--" to pnpm or arguments won't be parsed correctly so for this command only we are gonna treat pnpm as npm equivalent/
    const runTauriArgs =
      packageManager === 'yarn' || argv.binary
        ? ['tauri', 'init']
        : ['run', 'tauri', '--', 'init']

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
      answers: recipeAnswers ?? {}
    })
  }
}

function logStep(msg: string): void {
  const out = `${green('>>')} ${bold(cyan(msg))}`
  console.log(out)
}
