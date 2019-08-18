const appPaths = require('./app-paths')

module.exports = cfg => {
  const tauriConf = require(appPaths.resolve.app('tauri.conf.js'))(cfg.ctx)
  return Object.assign({
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
}
