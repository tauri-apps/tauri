module.exports = function () {
  return {
    build: {
      distDir: '', // must be an absolute folder path for now
      APP_URL: ''  // assumes an localhost server for now
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
        all: 'false'
      },
      window: {
        title: 'Tauri App'
      },
      security: {
        csp: 'default-src data: filesystem: ws: \'unsafe-eval\' \'unsafe-inline\''
      }
    }
  }
}
