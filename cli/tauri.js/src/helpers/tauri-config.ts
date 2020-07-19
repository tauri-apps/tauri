import { existsSync, readFileSync } from 'fs-extra'
import { TauriConfig } from 'types'
import { merge } from 'webpack-merge'
import logger from '../helpers/logger'
import * as appPaths from './app-paths'
import nonWebpackRequire from '../helpers/non-webpack-require'
import chalk from 'chalk'
import { isTauriConfig, ajv } from '../types/config.validator'

const error = logger('ERROR:', chalk.red)

const getTauriConfig = (cfg: Partial<TauriConfig>): TauriConfig => {
  const pkgPath = appPaths.resolve.app('package.json')
  const tauriConfPath = appPaths.resolve.tauri('tauri.conf.json')
  if (!existsSync(tauriConfPath)) {
    error(
      "Could not find a tauri config (tauri.conf.json) in your app's directory."
    )
    process.exit(1)
  }
  const tauriConf = JSON.parse(readFileSync(tauriConfPath).toString()) as TauriConfig
  const pkg = existsSync(pkgPath) ? nonWebpackRequire(pkgPath) as { productName: string } : null

  const config = merge(
    {
      build: {},
      ctx: {},
      tauri: {
        embeddedServer: {
          active: true
        },
        bundle: {
          active: true,
          icon: [],
          resources: [],
          externalBin: [],
          deb: {
            depends: []
          },
          osx: {
            frameworks: []
          }
        },
        whitelist: {
          all: false
        },
        window: {
          title: pkg?.productName ?? 'Tauri App'
        },
        security: {
          csp: "default-src blob: data: filesystem: ws: http: https: 'unsafe-eval' 'unsafe-inline'"
        },
        inliner: {
          active: true
        }
      }
    } as any,
    tauriConf as any,
    cfg as any
  ) as TauriConfig

  if (!isTauriConfig(config)) {
    const messages = ajv.errorsText(
      isTauriConfig.errors?.filter(e => e.keyword !== 'if').map(e => {
        e.dataPath = e.dataPath.replace(/\./g, ' > ')
        if (e.keyword === 'additionalProperties' && typeof e.message === 'string' && 'additionalProperty' in e.params) {
          e.message = `has unknown property ${e.params.additionalProperty}`
        }
        return e
      }), { dataVar: 'tauri.conf.json', separator: '\n' }
    ).split('\n')

    for (const message of messages) {
      error(message)
    }
    process.exit(1)
  }

  const runningDevServer = config.build.devPath?.startsWith('http')
  if (!runningDevServer) {
    config.build.devPath = appPaths.resolve.tauri(config.build.devPath)
    process.env.TAURI_DIST_DIR = config.build.devPath
  }
  if (config.build.distDir) {
    config.build.distDir = appPaths.resolve.tauri(config.build.distDir)
    process.env.TAURI_DIST_DIR = config.build.distDir
  }

  // OSX bundle config
  if (config.tauri.bundle.osx) {
    const license = config.tauri.bundle.osx.license
    if (typeof license === 'string') {
      config.tauri.bundle.osx.license = appPaths.resolve.tauri(license)
    } else if (license !== null) {
      const licensePath = appPaths.resolve.app('LICENSE')
      if (existsSync(licensePath)) {
        config.tauri.bundle.osx.license = licensePath
      }
    }
  }

  // bundle targets
  if (Array.isArray(config.tauri.bundle.targets)) {
    if (process.platform !== 'win32') {
      config.tauri.bundle.targets = config.tauri.bundle.targets.filter(t => t !== 'msi')
    }
  }

  process.env.TAURI_DIR = appPaths.tauriDir
  process.env.TAURI_CONFIG = JSON.stringify(config)

  return config
}

export default getTauriConfig
