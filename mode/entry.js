const compileTemplate = require('lodash.template'),
  { readFileSync, writeFileSync, ensureDir } = require('fs-extra'),
  path = require('path')

module.exports.generate = (outDir, cfg, ctx, tauri = false) => {
  const apiTemplate = readFileSync(path.resolve(__dirname, '../lib/tauri.js'), 'utf-8')
  const apiContent = compileTemplate(apiTemplate)({
    ...cfg,
    ctx: ctx,
    confName: `${tauri ? 'tauri' : 'quasar'}.conf.js`
  })
  ensureDir(outDir).then(() => {
    writeFileSync(path.join(outDir, 'tauri.js'), apiContent, 'utf-8')
  })
}
