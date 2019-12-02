const path = require('path')
const distDir = path.resolve(__dirname, './dist')

module.exports = function () {
  return {
    build: {
      distDir: distDir,
      devPath: 'http://localhost:7333' // devServer URL or path to html file
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
      },
      automaticStart: {
        active: true
      }
    }
  }
}
