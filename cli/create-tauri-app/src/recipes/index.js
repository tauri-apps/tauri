const fs = require("fs");
const path = require("path");
const { installDependencies } = require("tauri/dist/api/dependency-manager")

const applyRecipe = (args) => {
  if(args["recipe"]){
    const filename = `./${args["recipe"]}.js`;
    const filepath = path.join(__dirname, filename);

    if(fs.existsSync(filepath)){
      return require(filename)(args);
    } else {
      console.log("No such recipe for Tauri.");
      process.exit(1);

      return false;
    }
  } else {
    console.log("Using default `vanilla` recipe.")
    return require("./vanilla")(args);
  }
}

module.exports = async (args) => {
  const { appDir } = await applyRecipe(args)

  process.chdir(appDir);

  let hasYarn = false;
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

  await installDependencies()
}