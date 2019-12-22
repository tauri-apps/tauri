const path = require('path')
const distDir = path.resolve(__dirname, './dist/spa')

module.exports = function () {
  return {
    build: {
      distDir: distDir,
      devPath: 'http://localhost:7334' // devServer URL or html dir
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
        all: true
      },
      window: {
        title: 'Tauri App'
      },
      security: {
        csp: 'default-src data: filesystem: ws: http: https: \'unsafe-eval\' \'unsafe-inline\''
      },
      edge: {
        active: true
      }
    }
  }
}
