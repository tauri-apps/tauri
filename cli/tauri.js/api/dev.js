module.exports = config => {
  const { tauriDir } = require('../helpers/app-paths')
  const Runner = require('../runner')
  const merge = require('webpack-merge')
  const tauri = new Runner()
  const tauriConfig = require('../helpers/tauri-config')(
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
