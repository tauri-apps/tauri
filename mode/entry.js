const compileTemplate = require('lodash.template'),
  { readFileSync, writeFileSync
  } = require('fs'),
  path = require('path')

module.exports.generate = (outDir, cfg, tauri = false) => {
  const apiTemplate = readFileSync(path.resolve(__dirname, '../../lib/tauri.js'), 'utf-8')
  const apiContent = compileTemplate(apiTemplate)({
    ...cfg,
    confName: `${tauri ? 'tauri' : 'quasar'}.conf.js`
  })
  writeFileSync(path.join(outDir, 'tauri.js'), apiContent, 'utf-8')
}
