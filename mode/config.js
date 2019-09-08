const init = {
  embeddedServer: {},
  bundle: {},
  whitelist: {},
  window: {},
  security: {}
}

const defaultObject = {
  embeddedServer: {
    active: true
  },
  bundle: {
    active: true
  },
  whitelist: {},
  window: {
    title: 'Tauri App'
  },
  security: {
    csp: 'default-src data: filesystem: ws: http: https: \'unsafe-eval\' \'unsafe-inline\''
  }
}

module.exports.init = init
module.exports.defaultObject = defaultObject
