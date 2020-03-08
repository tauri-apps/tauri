import Inliner from '@tauri-apps/tauri-inliner'
import toml from '@tauri-apps/toml'
import chokidar, { FSWatcher } from 'chokidar'
import { existsSync, readFileSync, writeFileSync } from 'fs-extra'
import { JSDOM } from 'jsdom'
import { debounce, template } from 'lodash'
import path from 'path'
import * as entry from './entry'
import { appDir, tauriDir } from './helpers/app-paths'
import logger from './helpers/logger'
import onShutdown from './helpers/on-shutdown'
import { spawn } from './helpers/spawn'
const getTauriConfig = require('./helpers/tauri-config')
import { TauriConfig } from './types/config'

const log = logger('app:tauri', 'green')
const warn = logger('app:tauri (template)', 'red')

class Runner {
  pid: number
  tauriWatcher?: FSWatcher
  devPath?: string
  killPromise?: Function

  constructor() {
    this.pid = 0
    this.tauriWatcher = undefined
    onShutdown(() => {
      this.stop().catch(e => {
        throw e
      })
    })
  }

  async run(cfg: TauriConfig): Promise<void> {
    const devPath = cfg.build.devPath

    if (this.pid) {
      if (this.devPath !== devPath) {
        await this.stop()
      } else {
        return
      }
    }

    this.__manipulateToml(toml => {
      this.__whitelistApi(cfg, toml)
    })

    entry.generate(tauriDir, cfg)

    const runningDevServer = devPath.startsWith('http')
    let inlinedAssets: string[] = []

    if (!runningDevServer) {
      inlinedAssets = await this.__parseHtml(cfg, devPath)
    }

    process.env.TAURI_INLINED_ASSSTS = inlinedAssets.join('|')

    this.devPath = devPath

    const features = runningDevServer ? ['dev-server'] : []

    const startDevTauri = async (): Promise<void> => {
      return this.__runCargoCommand({
        cargoArgs: ['run'].concat(
          features.length ? ['--features', ...features] : []
        ),
        dev: true
      })
    }

    // Start watching for tauri app changes
    // eslint-disable-next-line security/detect-non-literal-fs-filename
    this.tauriWatcher = chokidar
      .watch(
        [
          path.join(tauriDir, 'src'),
          path.join(tauriDir, 'Cargo.toml'),
          path.join(tauriDir, 'build.rs'),
          path.join(appDir, 'tauri.conf.json')
        ],
        {
          ignoreInitial: true
        }
      )
      .on(
        'change',
        debounce((path: string) => {
          this.__stopCargo()
            .then(() => {
              if (path.includes('tauri.conf.json')) {
                this.run(getTauriConfig({ ctx: cfg.ctx })).catch(e => {
                  throw e
                })
              } else {
                startDevTauri().catch(e => {
                  throw e
                })
              }
            })
            .catch(err => {
              warn(err)
              process.exit(1)
            })
        }, 1000)
      )

    return startDevTauri()
  }

  async build(cfg: TauriConfig): Promise<void> {
    this.__manipulateToml(toml => {
      this.__whitelistApi(cfg, toml)
    })

    entry.generate(tauriDir, cfg)

    const inlinedAssets = await this.__parseHtml(cfg, cfg.build.distDir)
    process.env.TAURI_INLINED_ASSSTS = inlinedAssets.join('|')

    const features = [
      cfg.tauri.embeddedServer.active ? 'embedded-server' : 'no-server'
    ]

    const buildFn = async (target?: string): Promise<void> =>
      this.__runCargoCommand({
        cargoArgs: [
          cfg.tauri.bundle.active ? 'tauri-bundler' : 'build',
          '--features',
          ...features
        ]
          .concat(cfg.ctx.debug ? [] : ['--release'])
          .concat(target ? ['--target', target] : [])
      })

    if (!cfg.ctx.target) {
      // if no target specified,
      // build only for the current platform
      await buildFn()
    } else {
      const targets = cfg.ctx.target.split(',')

      for (const target of targets) {
        await buildFn(target)
      }
    }
  }

  async __parseHtml(cfg: TauriConfig, indexDir: string): Promise<string[]> {
    const inlinedAssets: string[] = []

    return new Promise((resolve, reject) => {
      const indexPath = path.join(indexDir, 'index.html')
      if (!existsSync(indexPath)) {
        warn(
          `Error: cannot find index.html in "${indexDir}". Did you forget to build your web code or update the build.distDir in tauri.conf.json?`
        )
        reject(new Error('Could not find index.html in dist dir.'))
      }

      const rewriteHtml = (html: string, interceptor?: (dom: JSDOM) => void) => {
        const dom = new JSDOM(html)
        const document = dom.window.document
        if (interceptor !== undefined) {
          interceptor(dom)
        }

        if (!((cfg.ctx.dev && cfg.build.devPath.startsWith('http')) || cfg.tauri.embeddedServer.active)) {
          const mutationObserverTemplate = require('!!raw-loader!!../templates/mutation-observer').default
          const compiledMutationObserver = template(mutationObserverTemplate)

          const bodyMutationObserverScript = document.createElement('script')
          bodyMutationObserverScript.text = compiledMutationObserver({
            target: 'body',
            inlinedAssets: JSON.stringify(inlinedAssets)
          })
          document.body.insertBefore(bodyMutationObserverScript, document.body.firstChild)

          const headMutationObserverScript = document.createElement('script')
          headMutationObserverScript.text = compiledMutationObserver({
            target: 'head',
            inlinedAssets: JSON.stringify(inlinedAssets)
          })
          document.head.insertBefore(headMutationObserverScript, document.head.firstChild)
        }

        const tauriScript = document.createElement('script')
        // @ts-ignore
        tauriScript.text = readFileSync(path.join(tauriDir, 'tauri.js'))
        document.body.insertBefore(tauriScript, document.body.firstChild)

        const csp = cfg.tauri.security.csp
        if (csp) {
          const cspTag = document.createElement('meta')
          cspTag.setAttribute('http-equiv', 'Content-Security-Policy')
          cspTag.setAttribute('content', csp)
          document.head.appendChild(cspTag)
        }
        writeFileSync(
          path.join(indexDir, 'index.tauri.html'),
          dom.serialize()
        )
      }

      const domInterceptor = cfg.tauri.embeddedServer.active ? undefined : (dom: JSDOM) => {
        const document = dom.window.document
        document.querySelectorAll('link').forEach((link: HTMLLinkElement) => {
          link.removeAttribute('rel')
          link.removeAttribute('as')
        })
      }

      if (cfg.tauri.embeddedServer.active || !cfg.tauri.inliner.active) {
        rewriteHtml(readFileSync(indexPath).toString(), domInterceptor)
        resolve(inlinedAssets)
      } else {
        new Inliner(indexPath, (err: Error, html: string) => {
          if (err) {
            reject(err)
          } else {
            rewriteHtml(html, domInterceptor)
            resolve(inlinedAssets)
          }
        }).on('progress', (event: string) => {
          const match = event.match(/([\S\d]+)\.([\S\d]+)/g)
          match && inlinedAssets.push(match[0])
        })
      }
    })
  }

  async stop(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.tauriWatcher && this.tauriWatcher.close()
      this.__stopCargo()
        .then(resolve)
        .catch(reject)
    })
  }

  async __runCargoCommand({
    cargoArgs,
    extraArgs,
    dev = false
  }: {
    cargoArgs: string[]
    extraArgs?: string[]
    dev?: boolean
  }): Promise<void> {
    return new Promise((resolve, reject) => {
      this.pid = spawn(
        'cargo',

        extraArgs ? cargoArgs.concat(['--']).concat(extraArgs) : cargoArgs,

        tauriDir,

        code => {
          if (code) {
            warn()
            warn('⚠️  [FAIL] Cargo CLI has failed')
            warn()
            reject()
            process.exit(1)
          } else if (!dev) {
            resolve()
          }

          if (this.killPromise) {
            this.killPromise()
            this.killPromise = undefined
          } else if (dev) {
            warn()
            warn('Cargo process was killed. Exiting...')
            warn()
            process.exit(0)
          }

          resolve()
        }
      )
      
      if (dev) {
        resolve()
      }
    })
  }

  async __stopCargo(): Promise<void> {
    const pid = this.pid

    if (!pid) {
      return Promise.resolve()
    }

    log('Shutting down tauri process...')
    this.pid = 0

    return new Promise((resolve, reject) => {
      this.killPromise = resolve
      try {
        process.kill(pid)
      } catch (e) {
        reject(e)
      }
    })
  }

  __manipulateToml(callback: (tomlContents: object) => void): void {
    const tomlPath = path.join(tauriDir, 'Cargo.toml')
    const tomlFile = readFileSync(tomlPath)
    // @ts-ignore
    const tomlContents = toml.parse(tomlFile)

    callback(tomlContents)

    const output = toml.stringify(tomlContents)
    writeFileSync(tomlPath, output)
  }

  __whitelistApi(
    cfg: TauriConfig,
    tomlContents: { [index: string]: any }
  ): void {
    const tomlFeatures = []

    if (cfg.tauri.whitelist.all) {
      tomlFeatures.push('all-api')
    } else {
      const toKebabCase = (value: string): string => {
        return value.replace(/([a-z])([A-Z])/g, "$1-$2")
          .replace(/\s+/g, '-')
          .toLowerCase()
      }
      const whitelist = Object.keys(cfg.tauri.whitelist).filter(
        w => cfg.tauri.whitelist[String(w)] === true
      )
      tomlFeatures.push(...whitelist.map(toKebabCase))
    }

    if (cfg.tauri.edge.active) {
      tomlFeatures.push('edge')
    }

    tomlContents.dependencies.tauri.features = tomlFeatures
  }
}

export default Runner
