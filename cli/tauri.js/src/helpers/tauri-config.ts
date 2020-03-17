import { existsSync } from 'fs-extra'
import { resolve } from 'path'
import { TauriConfig } from 'types'
import merge from 'webpack-merge'
import logger from '../helpers/logger'
import * as appPaths from './app-paths'
import nonWebpackRequire from '../helpers/non-webpack-require'

const error = logger('ERROR:', 'red')

const getTauriConfig = (cfg: Partial<TauriConfig>): TauriConfig => {
  const pkgPath = appPaths.resolve.app('package.json')
  const tauriConfPath = appPaths.resolve.tauri('tauri.conf.json')
  if (!existsSync(pkgPath)) {
    error("Could not find a package.json in your app's directory.")
    process.exit(1)
  }
  if (!existsSync(tauriConfPath)) {
    error(
      "Could not find a tauri config (tauri.conf.json) in your app's directory."
    )
    process.exit(1)
  }
  const tauriConf = nonWebpackRequire(tauriConfPath)
  const pkg = nonWebpackRequire(pkgPath)

  const config = merge(
    {
      build: {},
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
          title: pkg.productName
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
    } as any,
    tauriConf,
    cfg as any
  ) as TauriConfig

  const runningDevServer = config.build.devPath && config.build.devPath.startsWith('http')
  if (!runningDevServer) {
    config.build.devPath = appPaths.resolve.tauri(config.build.devPath)
    process.env.TAURI_DIST_DIR = appPaths.resolve.app(config.build.devPath)
  }
  if (config.build.distDir) {
    config.build.distDir = appPaths.resolve.tauri(config.build.distDir)
    process.env.TAURI_DIST_DIR = appPaths.resolve.app(config.build.distDir)
  }
  
  if (!process.env.TAURI_DIST_DIR) {
    error("Couldn't resolve the dist dir. Make sure you have `devPath` or `distDir` under tauri.conf.json > build")
    process.exit(1)
  }

  process.env.TAURI_DIR = appPaths.tauriDir
  process.env.TAURI_CONFIG = JSON.stringify(config)

  return config
}

export default getTauriConfig
