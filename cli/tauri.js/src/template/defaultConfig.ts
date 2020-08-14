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
      targets: 'all', // or an array of targets
      identifier: 'com.tauri.dev',
      icon: [
        'icons/32x32.png',
        'icons/128x128.png',
        'icons/128x128@2x.png',
        'icons/icon.icns',
        'icons/icon.ico'
      ],
      resources: [],
      externalBin: [],
      copyright: '',
      category: 'DeveloperTool',
      shortDescription: '',
      longDescription: '',
      deb: {
        depends: [],
        useBootstrapper: false
      },
      osx: {
        frameworks: [],
        minimumSystemVersion: '',
        useBootstrapper: false
      },
      exceptionDomain: ''
    },
    allowlist: {
      all: true
    },
    window: {
      title: 'Tauri App',
      width: 800,
      height: 600,
      resizable: true,
      fullscreen: false
    },
    security: {
      csp:
        "default-src blob: data: filesystem: ws: http: https: 'unsafe-eval' 'unsafe-inline'"
    },
    inliner: {
      active: true
    }
  }
}
