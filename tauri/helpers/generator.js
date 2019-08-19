const compileTemplate = require('lodash.template'),
  { readFileSync, writeFileSync } = require('fs'),
  appPaths = require('./app-paths'),
  path = require('path')

module.exports = cfg => {
  const apiTemplate = readFileSync(path.resolve(__dirname, '../../lib/tauri.js'), 'utf-8')
  const apiContent = compileTemplate(apiTemplate)({
    ...cfg,
    confName: 'tauri.conf.js'
  })
  writeFileSync(appPaths.resolve.tauri('tauri.js'), apiContent, 'utf-8')
}
