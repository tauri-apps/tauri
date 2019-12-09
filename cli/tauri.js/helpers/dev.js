module.exports = config => {
  const { tauriDir } = require('./app-paths')
  const Runner = require('../runner')
  const merge = require('webpack-merge')
  const tauri = new Runner()
  const tauriConfig = require('./tauri-config')(
    merge(
      {
        ctx: {
          debug: true
        }
      },
      config
    )
  )

  require('../generator').generate(tauriConfig.tauri)
  require('../entry').generate(tauriDir, tauriConfig)

  return tauri.run(tauriConfig)
}
