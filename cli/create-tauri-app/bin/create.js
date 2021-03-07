const recipes = require("../src/recipes");

module.exports = async (args) => {
  await recipes(args);
}
