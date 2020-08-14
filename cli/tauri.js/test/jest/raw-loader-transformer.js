module.exports = {
  process: (content) => `module.exports = {default: ${JSON.stringify(content)}}`
};
