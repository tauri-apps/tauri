const 
  path = require('path'),
  { writeFileSync } = require('fs')

module.exports.generate = tauriConfig => {
  const 
    { bundle, ...cfg } = tauriConfig,
    outDir = path.resolve(__dirname, '../..')
  writeFileSync(path.join(outDir, 'config.json'), JSON.stringify(cfg))
  writeFileSync(path.join(outDir, 'bundle.json'), JSON.stringify(bundle))
}
