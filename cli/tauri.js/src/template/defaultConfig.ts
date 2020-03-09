export default {
  build: {
    distDir: '../dist',
    devPath: 'http://localhost:4000',
    beforeDevCommand: '',
    beforeBuildCommand: ''
  },
  ctx: {},
  tauri: {
    embeddedServer: {
      active: true
    },
    bundle: {
      active: true,
      identifier: 'com.tauri.dev',
      icon: ['icons/32x32.png', 'icons/128x128.png', 'icons/128x128@2x.png', 'icons/icon.icns', 'icons/icon.ico'],
      resources: [],
      externalBin: [],
      copyright: '',
      category: '',
      shortDescription: '',
      longDescription: '',
      deb: {
        depends: []
      },
      osx: {
        frameworks: [],
        minimumSystemVersion: ''
      },
      exceptionDomain: ''
    },
    whitelist: {
      all: true
    },
    window: {
      title: 'Tauri App'
    },
    security: {
      csp: "default-src blob: data: filesystem: ws: http: https: 'unsafe-eval' 'unsafe-inline'"
    },
    edge: {
      active: true
    },
    inliner: {
      active: true
    }
  }
}
