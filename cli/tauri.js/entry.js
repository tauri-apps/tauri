const compileTemplate = require('lodash.template')
const { readFileSync, writeFileSync, ensureDirSync } = require('fs-extra')
const path = require('path')

module.exports.generate = (outDir, cfg) => {
  // this MUST be from the templates repo
  const apiTemplate = readFileSync(path.resolve(__dirname, './templates/tauri.js'), 'utf-8')
  const apiContent = compileTemplate(apiTemplate)(cfg)

  ensureDirSync(outDir)
  writeFileSync(path.join(outDir, 'tauri.js'), apiContent, 'utf-8')
}
