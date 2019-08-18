const compileTemplate = require('lodash.template'),
  fs = require('fs'),
  appPaths = require('./app-paths'),
  path = require('path')

module.exports = cfg => {
  const apiTemplate = fs.readFileSync(path.resolve(__dirname, '../../lib/tauri.js'), 'utf-8')
  const apiContent = compileTemplate(apiTemplate)({
    ...cfg,
    confName: 'tauri.conf.js'
  })
  fs.writeFileSync(appPaths.resolve.tauri('tauri.js'), apiContent, 'utf-8')
}
