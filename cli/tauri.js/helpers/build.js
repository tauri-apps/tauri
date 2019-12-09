module.exports = config => {
  const { tauriDir } = require('./app-paths')
  const merge = require('webpack-merge')
  const Runner = require('../runner')
  const tauri = new Runner({ modeDir: tauriDir })
  const tauriConfig = require('./tauri-config')(
    merge(
      {
        ctx: {
          prod: true
        }
      },
      config
    )
  )

  require('../generator').generate(tauriConfig.tauri)
  require('../entry').generate(tauriDir, tauriConfig)

  return tauri.build(tauriConfig)
}
