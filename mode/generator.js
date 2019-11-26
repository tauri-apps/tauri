const
  path = require('path')
const { writeFileSync } = require('fs-extra')
const { tauriDir } = require('./helpers/app-paths')

module.exports.generate = tauriConfig => {
  const
    { bundle, ...cfg } = tauriConfig
  const outDir = tauriDir
  writeFileSync(path.join(outDir, 'config.json'), JSON.stringify(cfg))
  writeFileSync(path.join(outDir, 'bundle.json'), JSON.stringify(bundle))
}
