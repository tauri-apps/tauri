async function run() {
  const {
    installDependencies,
    updateDependencies
  } = require('../dist/api/dependency-manager')

  const choice = process.argv[2]
  if (choice === 'install') {
    await installDependencies()
  } else if (choice === 'update') {
    await updateDependencies()
  } else {
    console.log(`
    Description
      Tauri dependency management script
    Usage
      $ tauri deps [install|update]
  `)
  }
}

run()
