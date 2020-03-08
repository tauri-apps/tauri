// TODO: Clean up types, properly mark which ones are optional
// May need to have different types for each stage of config generation process

export interface TauriConfig {
  build: {
    distDir: string
    devPath: string
  }
  ctx: {
    prod?: boolean
    dev?: boolean
    target: string
    debug?: boolean
    targetName: string
    exitOnPanic?: boolean
  }
  bundle: {}
  tauri: {
    inlinedAssets: string[]
    devPath: string
    embeddedServer: {
      active: boolean
    }
    bundle: {
      active: boolean
    }
    whitelist: {
      all: boolean
      [index: string]: boolean
    }
    window: {
      title: string
      width: number
      height: number
      resizable: boolean
    }
    security: {
      csp: string
    }
    edge: {
      active: boolean
    }
    inliner: {
      active: boolean
    }
  }
}
