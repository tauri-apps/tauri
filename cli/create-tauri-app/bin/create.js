const fs = require("fs");
const scaffe = require("scaffe");
const init = require("tauri/dist/api/init");
const { execSync } = require("child_process");
const recipes = require("../src/recipes");

module.exports = (args) => {
  const appName = args["_"][0];
  const recipeConfig = recipes(args);
  const {
    templateDir,
    customConfig,
  } = recipeConfig;

  scaffe.generate(templateDir, appName, {name: appName}, async (err) => {
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
          },
        },
        ...customConfig,
      },
    })

    process.chdir(appName)

    let hasYarn = false
    try {
      execSync("yarn version", {stdio: "ignore"})
      hasYarn = true
    } catch (error) {
      // Do nothing, the user doesn't have Yarn
    }
    // TODO add CLI arg to force npm usage
    if (hasYarn) {
      // Create empty yarn.lock so that installDependencies uses Yarn
      fs.writeFileSync("yarn.lock", "")
    }

    const { installDependencies } = require("tauri/dist/api/dependency-manager")
    await installDependencies()
  })
}
