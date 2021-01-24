const path = require("path");
const scaffe = require("scaffe");
const init = require("tauri/dist/api/init");

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

    const { installDependencies } = require('tauri/dist/api/dependency-manager')
    await installDependencies()
  })
}
