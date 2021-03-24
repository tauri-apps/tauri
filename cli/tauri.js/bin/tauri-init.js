const { runOnRustCli } = require('../dist/helpers/rust-cli')

/**
 * init creates the src-tauri folder
 */
function main(cliArgs) {
  runOnRustCli('init', cliArgs).promise.then(() => {
    const { installDependencies } = require('../dist/api/dependency-manager')
    return installDependencies()
  })
}

module.exports = main
