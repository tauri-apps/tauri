const fs = require("fs");
const path = require("path");

module.exports = (args) => {
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
  }

  return require("./vanilla")(args);
}