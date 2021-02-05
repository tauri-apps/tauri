const path = require("path");

module.exports = (args) => {
  const templateDir = path.join(__dirname, "../templates/vanilla");

  const config = {
    templateDir,
    customConfig: {
      build: {
        devPath: "../dist"
      }
    }
  };

  return config;
}