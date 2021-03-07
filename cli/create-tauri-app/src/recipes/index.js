const fs = require("fs");
const path = require("path");

const applyRecipe = async (args) => {
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

    return await require("./vanilla")(args);;
  }
}

module.exports = async (args) => {
  const { output } = await applyRecipe(args)
  console.log(output)
}
