const appPaths = require('./app-paths')
const merge = require('webpack-merge')

module.exports = cfg => {
  const tauriConf = require(appPaths.resolve.app('tauri.conf.js'))(cfg.ctx)
  const config = merge({
    build: {
      distDir: './dist'
    },
    ctx: {},
    tauri: {
      embeddedServer: {
        active: true
      },
      bundle: {
        active: true
      },
      whitelist: {
        all: false
      },
      window: {
        title: require(appPaths.resolve.app('package.json')).productName
      },
      security: {
        csp: 'default-src data: filesystem: ws: http: https: \'unsafe-eval\' \'unsafe-inline\''
      },
      automaticStart: {
        active: false,
        devArgs: [],
        buildArgs: []
      },
      edge: {
        active: true
      }
    }
  }, tauriConf, cfg)

  process.env.TAURI_DIST_DIR = appPaths.resolve.app(config.build.distDir)
  process.env.TAURI_DIR = appPaths.tauriDir

  return config
}
