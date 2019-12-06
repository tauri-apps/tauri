const appPaths = require('./app-paths')
const merge = require('webpack-merge')
const error = require('../helpers/logger')('ERROR:', 'red')
const { existsSync } = require('fs-extra')

module.exports = cfg => {
  const pkgPath = appPaths.resolve.app('package.json')
  const tauriConfPath = appPaths.resolve.app('tauri.conf.js')
  if (!existsSync(pkgPath)) {
    error('Could not find a package.json in your app\'s directory.')
    process.exit(1)
  }
  if (!existsSync(tauriConfPath)) {
    error('Could not find a tauri config (tauri.conf.js) in your app\'s directory.')
    process.exit(1)
  }
  const tauriConf = require(tauriConfPath)(cfg.ctx)
  const pkg = require(pkgPath)

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
        title: pkg.productName
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
  process.env.TAURI_CONFIG_DIR = appPaths.tauriDir

  return config
}
