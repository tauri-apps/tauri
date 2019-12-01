const path = require('path')
const distDir = path.resolve(__dirname, './dist/spa')

module.exports = function () {
  return {
    build: {
      distDir: distDir,
      devPath: 'http://localhost:8080' // devServer URL or path to html file
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
