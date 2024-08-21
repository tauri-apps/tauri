const configPrettier = {
  overrides: [{ files: ['.changes/**.md'], options: { singleQuote: false } }],
  semi: false,
  singleQuote: true,
  trailingComma: 'none'
}

module.exports = configPrettier
