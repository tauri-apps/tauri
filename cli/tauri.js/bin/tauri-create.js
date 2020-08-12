const parseArgs = require('minimist')
const inquirer = require('inquirer')
const { resolve } = require('path')
const { merge } = require('lodash')
const { 
  recipeShortNames, 
  recipeDescriptiveNames, 
  recipeByDescriptiveName, 
  recipeByShortName 
} = require('../dist/api/recipes')

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
 * @property {string} r
 * @property {string} recipe
 */
function main(cliArgs) {
  const argv = parseArgs(cliArgs, {
    alias: {
      h: 'help',
      f: 'force',
      l: 'log',
      d: 'directory',
      t: 'tauri-path',
      A: 'app-name',
      W: 'window-title',
      D: 'dist-dir',
      P: 'dev-path',
      r: 'recipe'
    },
    boolean: ['h', 'l', 'ci']
  })

  if (argv.help) {
    printUsage()
    return 0
  }

  if (argv.ci) {
    runInit(argv)
  } else {
    getOptionsInteractive(argv).then(responses => runInit(argv, responses))
  }
}

function printUsage() {
  console.log(`
  Description
    Inits the Tauri template. If Tauri cannot find the tauri.conf.json
    it will create one.
  Usage
    $ tauri create
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
    --recipe, -r         Add UI framework recipe. None by default. 
                         Supported recipes: [${recipeShortNames.join('|')}]
    `)
}

const getOptionsInteractive = (argv) => {
  let defaultAppName = argv.A
  if (!defaultAppName) {
    try {
      const packageJson = JSON.parse(readFileSync(resolve(process.cwd(), 'package.json')).toString())
      defaultAppName = packageJson.displayName || packageJson.name
    } catch {}
  }

  return inquirer
    .prompt([{
        type: 'input',
        name: 'appName',
        message: 'What is your app name?',
        default: defaultAppName
      }, {
        type: 'input',
        name: 'tauri.window.title',
        message: 'What should the window title be?',
        default: 'Tauri App',
        when: () => !argv.W
      },
      {
        type: 'list',
        name: 'recipeName',
        message: 'Would you like to add a UI recipe?',
        choices: recipeDescriptiveNames,
        default: 'No recipe',
        when: () => !argv.r
      }
    ])
    .then(answers =>
      inquirer
        .prompt([
          {
            type: 'input',
            name: 'build.devPath',
            message: 'What is the url of your dev server?',
            default: 'http://localhost:4000',
            when: () => !argv.P && !argv.p && answers.recipeName === 'No recipe' || argv.r === 'none'
          },
          {
            type: 'input',
            name: 'build.distDir',
            message: 'Where are your web assets (HTML/CSS/JS) located, relative to the "<current dir>/src-tauri" folder that will be created?',
            default: '../dist',
            when: () => !argv.D && answers.recipeName === 'No recipe' || argv.r === 'none'
          }
        ])
        .then(answers2 => ({...answers, ...answers2}))
    )
    .catch(error => {
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
  const {
    appName,
    recipeName,
    ...configOptions
  } = config
  const init = require('../dist/api/init')

  let recipe;
  let recipeSelection = 'none'

  if (recipeName !== undefined) {
    recipe = recipeByDescriptiveName(recipeName)
  } else if (argv.r) {
    recipe = recipeByShortName(argv.r)
  }

  let buildConfig = {
    distDir: argv.D,
    devPath: argv.P
  }

  if (recipe !== undefined) {
    recipeSelection = recipe.shortName
    buildConfig = recipe.configUpdate(buildConfig)
  }

  const directory = argv.d || process.cwd()
  
  init({
    directory,
    force: argv.f || null,
    logging: argv.l || null,
    tauriPath: argv.t || null,
    appName: appName || argv.A || null,
    customConfig: merge(configOptions, {
      build: buildConfig,
      tauri: {
        window: {
          title: argv.W
        }
      }
    })
  })
   
  const {
    installDependencies
  } = require('../dist/api/dependency-manager')
  await installDependencies()
  
  if (recipe !== undefined) {
    const {
      installRecipeDependencies,
      runRecipePostConfig
    } = require('../dist/api/recipes/install')
  
    await installRecipeDependencies(recipe, directory)
    await runRecipePostConfig(recipe, directory)
  }
}

module.exports = main
