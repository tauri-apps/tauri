const path = require("path");
const scaffe = require("scaffe");
const init = require("tauri/dist/api/init");
const { execSync } = require('child_process')
const fs = require('fs')

module.exports = (appName, args) => {
  const template_dir = path.join(__dirname, "../templates/vanilla")

  scaffe.generate(template_dir, appName, {name: appName}, async (err) => {
    if(err){
      console.log(err)
    }

    init({
      directory: appName,
      force: args.f || null,
      logging: args.l || null,
      tauriPath: args.t || null,
      appName: appName || args.A || null,
      customConfig: {
        tauri: {
          window: {
            title: appName
          }
        }
      }
    })

    process.chdir(appName)

    let hasYarn = false
    try {
      execSync('yarn version', {stdio: "ignore"})
      hasYarn = true
    } catch (error) {
      // Do nothing, the user doesn't have Yarn
    }
    // TODO add CLI arg to force npm usage
    if (hasYarn) {
      // Create empty yarn.lock so that installDependencies uses Yarn
      fs.writeFileSync('yarn.lock', '')
    }

    const { installDependencies } = require('tauri/dist/api/dependency-manager')
    await installDependencies()
  })
}
