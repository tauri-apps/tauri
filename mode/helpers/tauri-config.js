const appPaths = require('./app-paths'),
  merge = require('webpack-merge')

module.exports = cfg => {
  const tauriConf = require(appPaths.resolve.app('tauri.conf.js'))(cfg.ctx)
  const config = merge({
    build: {},
    ctx: {},
    tauri: {
      embeddedServer: {
        active: true
      },
      bundle: {
        active: true
      },
      whitelist: {},
      window: {
        title: require(appPaths.resolve.app('package.json')).productName
      },
      security: {
        csp: 'default-src data: filesystem: ws: http: https: \'unsafe-eval\' \'unsafe-inline\''
      }
    }
  }, tauriConf, cfg)

  process.env.TAURI_DIST_DIR = appPaths.resolve.app(config.build.distDir)
  console.log(tauriConf)
  return config
}
