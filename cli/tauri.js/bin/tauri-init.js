const parseArgs = require('minimist')
const tauriCreate = require('./tauri-create')
const { recipeShortNames } = require('../dist/api/recipes')

/**
 * init is an alias for create -r none, same as 
 * creating a fresh tauri project with no UI recipe applied.
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
    },
    boolean: ['h']
  })

  if (argv.help) {
    printUsage()
    process.exit(0)
  }

  // delegate actual work to create command
  tauriCreate([...cliArgs, '-r', 'none'])
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
}

module.exports = main
