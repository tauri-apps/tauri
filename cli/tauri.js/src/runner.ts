import Inliner from '@tauri-apps/tauri-inliner'
import toml, { JsonMap } from '@tauri-apps/toml'
import chokidar, { FSWatcher } from 'chokidar'
import { existsSync, readFileSync, writeFileSync } from 'fs-extra'
import { JSDOM } from 'jsdom'
import { debounce, template } from 'lodash'
import path from 'path'
import * as entry from './entry'
import { tauriDir } from './helpers/app-paths'
import logger from './helpers/logger'
import onShutdown from './helpers/on-shutdown'
import { spawn, spawnSync } from './helpers/spawn'
import { TauriConfig } from './types/config'
import getTauriConfig from './helpers/tauri-config'

const log = logger('app:tauri', 'green')
const warn = logger('app:tauri (runner)', 'red')

class Runner {
  pid: number
  tauriWatcher?: FSWatcher
  devPath?: string
  killPromise?: Function
  ranBeforeDevCommand?: boolean

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

    if (!this.ranBeforeDevCommand && cfg.build.beforeDevCommand) {
      this.ranBeforeDevCommand = true // prevent calling it twice on recursive call on our watcher
      const [command, ...args] = cfg.build.beforeDevCommand.split(' ')
      spawnSync(command, args, tauriDir)
    }

    const tomlContents = this.__getManifest()
    this.__whitelistApi(cfg, tomlContents)
    this.__rewriteManifest(tomlContents)

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
        dev: true,
        exitOnPanic: cfg.ctx.exitOnPanic
      })
    }

    // Start watching for tauri app changes
    // eslint-disable-next-line security/detect-non-literal-fs-filename

    let tauriPaths: string[] = []
    // @ts-ignore
    if (tomlContents.dependencies.tauri.path) {
      // @ts-ignore
      const tauriPath = path.resolve(tauriDir, tomlContents.dependencies.tauri.path)
      tauriPaths = [
        tauriPath,
        `${tauriPath}-api`,
        `${tauriPath}-updater`,
        `${tauriPath}-utils`
      ]
    }

    // eslint-disable-next-line security/detect-non-literal-fs-filename
    this.tauriWatcher = chokidar
      .watch(
        [
          path.join(tauriDir, 'src'),
          path.join(tauriDir, 'Cargo.toml'),
          path.join(tauriDir, 'build.rs'),
          path.join(tauriDir, 'tauri.conf.json'),
          ...tauriPaths
        ].concat(runningDevServer ? [] : [devPath]),
        {
          ignoreInitial: true,
          ignored: runningDevServer ? null : path.join(devPath, 'index.tauri.html')
        }
      )
      .on(
        'change',
        debounce((changedPath: string) => {
          if (changedPath.startsWith(path.join(tauriDir, 'target'))) {
            return
          }
          (this.pid ? this.__stopCargo() : Promise.resolve())
            .then(() => {
              const shouldTriggerRun = changedPath.includes('tauri.conf.json') ||
                changedPath.startsWith(devPath)
              if (shouldTriggerRun) {
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
    if (cfg.build.beforeBuildCommand) {
      const [command, ...args] = cfg.build.beforeBuildCommand.split(' ')
      spawnSync(command, args, tauriDir)
    }

    const tomlContents = this.__getManifest()
    this.__whitelistApi(cfg, tomlContents)
    this.__rewriteManifest(tomlContents)

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

      const rewriteHtml = (html: string, interceptor?: (dom: JSDOM) => void): void => {
        const dom = new JSDOM(html)
        const document = dom.window.document
        if (interceptor !== undefined) {
          interceptor(dom)
        }

        // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
        if (!((cfg.ctx.dev && cfg.build.devPath.startsWith('http')) || cfg.tauri.embeddedServer.active)) {
          const mutationObserverTemplate = require('../templates/mutation-observer').default
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

      if ((!cfg.ctx.dev && cfg.tauri.embeddedServer.active) || !cfg.tauri.inliner.active) {
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
    dev = false,
    exitOnPanic = true
  }: {
    cargoArgs: string[]
    extraArgs?: string[]
    dev?: boolean
    exitOnPanic?: boolean
  }): Promise<void> {
    return new Promise((resolve, reject) => {
      this.pid = spawn(
        'cargo',

        extraArgs ? cargoArgs.concat(['--']).concat(extraArgs) : cargoArgs,

        tauriDir,

        code => {
          if (dev && !exitOnPanic && code === 101) {
            this.pid = 0
            resolve()
            return
          }

          if (code) {
            warn()
            warn('⚠️  [FAIL] Cargo CLI has failed')
            warn()
            reject(new Error('Cargo failed with status code ' + code.toString()))
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

  __getManifestPath(): string {
    return path.join(tauriDir, 'Cargo.toml')
  }

  __getManifest(): JsonMap {
    const tomlPath = this.__getManifestPath()
    const tomlFile = readFileSync(tomlPath).toString()
    const tomlContents = toml.parse(tomlFile)
    return tomlContents
  }

  __rewriteManifest(tomlContents: JsonMap): void {
    const tomlPath = this.__getManifestPath()
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
        return value.replace(/([a-z])([A-Z])/g, '$1-$2')
          .replace(/\s+/g, '-')
          .toLowerCase()
      }
      const whitelist = Object.keys(cfg.tauri.whitelist).filter(
        w => cfg.tauri.whitelist[String(w)]
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
