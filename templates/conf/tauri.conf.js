module.exports = function () {
  return {
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
        title: "Tauri App"
      },
      security: {
        csp: 'default-src data: filesystem: ws: http: https: \'unsafe-eval\' \'unsafe-inline\''
      }
    }
  }
}
