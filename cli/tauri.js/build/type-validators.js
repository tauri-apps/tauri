const { exec } = require("child_process")
const { readFileSync, writeFileSync } = require('fs')
const { resolve } = require('path')

const sourcePath = resolve(__dirname, '../src/types/config.ts')

exec(`typescript-json-validator --noExtraProps ${sourcePath} TauriConfig`, error => {
  if (error) {
    console.error(error.message)
    process.exit(error.code || 1)
  } else {
    const configValidatorPath = resolve(__dirname, '../src/types/config.validator.ts')
    const configValidator = readFileSync(configValidatorPath).toString()
    writeFileSync(configValidatorPath, configValidator.replace(`import Ajv = require('ajv');`, `import Ajv from 'ajv';`))
  }
})
