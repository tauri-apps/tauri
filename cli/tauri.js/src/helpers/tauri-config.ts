import { existsSync } from 'fs-extra'
import { TauriConfig } from 'types'
import merge from 'webpack-merge'
import logger from '../helpers/logger'
import * as appPaths from './app-paths'

const error = logger('ERROR:', 'red')

export default (cfg: Partial<TauriConfig>): TauriConfig => {
  const pkgPath = appPaths.resolve.app('package.json')
  const tauriConfPath = appPaths.resolve.app('tauri.conf.js')
  if (!existsSync(pkgPath)) {
    error("Could not find a package.json in your app's directory.")
    process.exit(1)
  }
  if (!existsSync(tauriConfPath)) {
    error(
      "Could not find a tauri config (tauri.conf.js) in your app's directory."
    )
    process.exit(1)
  }
  const tauriConf = __non_webpack_require__(tauriConfPath)(cfg.ctx)
  const pkg = __non_webpack_require__(pkgPath)

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
          csp:
            "default-src data: filesystem: ws: http: https: 'unsafe-eval' 'unsafe-inline'"
        },
        edge: {
          active: true
        }
      }
    } as any,
    tauriConf,
    cfg as any
  ) as TauriConfig

  process.env.TAURI_DIST_DIR = appPaths.resolve.app(config.build.distDir)
  process.env.TAURI_DIR = appPaths.tauriDir

  return config
}
