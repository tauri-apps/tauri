import Inliner from '@tauri-apps/tauri-inliner'
import toml, { JsonMap } from '@tauri-apps/toml'
import chokidar, { FSWatcher } from 'chokidar'
import { existsSync, readFileSync, writeFileSync } from 'fs-extra'
import { JSDOM } from 'jsdom'
import { debounce, template } from 'lodash'
import path from 'path'
import http from 'http'
import * as net from 'net'
import os from 'os'
import { findClosestOpenPort } from './helpers/net'
import { tauriDir, appDir } from './helpers/app-paths'
import logger from './helpers/logger'
import onShutdown from './helpers/on-shutdown'
import { spawn, spawnSync } from './helpers/spawn'
import { exec, ChildProcess } from 'child_process'
import { TauriConfig } from './types/config'
import { CargoManifest } from './types/cargo'
import getTauriConfig from './helpers/tauri-config'
import httpProxy from 'http-proxy'
import chalk from 'chalk'

const log = logger('app:tauri')
const warn = logger('app:tauri (runner)', chalk.red)

const WATCHER_INTERVAL = 1000

class Runner {
  pid: number
  rewritingToml: boolean = false
  tauriWatcher?: FSWatcher
  devPath?: string
  killPromise?: Function
  beforeDevProcess?: ChildProcess
  devServer?: net.Server

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
    let devPath = cfg.build.devPath

    if (this.pid) {
      if (this.devPath !== devPath) {
        await this.stop()
      }
    }

    if (!this.beforeDevProcess && cfg.build.beforeDevCommand) {
      log('Running `' + cfg.build.beforeDevCommand + '`')
      const ls = exec(cfg.build.beforeDevCommand, {
        cwd: appDir,
        env: process.env
      }, error => {
        if (error) {
          process.exit(1)
        }
      })

      ls.stderr?.pipe(process.stderr)
      ls.stdout?.pipe(process.stdout)
      this.beforeDevProcess = ls
    }

    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    const cargoManifest = this.__getManifest() as any as CargoManifest
    this.__whitelistApi(cfg, cargoManifest)
    this.__rewriteManifest(cargoManifest as unknown as toml.JsonMap)

    const runningDevServer = devPath.startsWith('http')

    let inlinedAssets: string[] = []

    if (runningDevServer) {
      // eslint-disable-next-line @typescript-eslint/no-this-alias
      const self = this
      const devUrl = new URL(devPath)
      const proxy = httpProxy.createProxyServer({
        ws: true,
        target: {
          host: devUrl.hostname,
          port: devUrl.port
        },
        selfHandleResponse: true
      })

      proxy.on('proxyRes', (proxyRes: http.IncomingMessage, req: http.IncomingMessage, res: http.ServerResponse) => {
        if (req.url === '/') {
          const body: Uint8Array[] = []
          proxyRes.on('data', (chunk: Uint8Array) => {
            body.push(chunk)
          })
          proxyRes.on('end', () => {
            const bodyStr = body.join('')
            const indexDir = os.tmpdir()
            writeFileSync(path.join(indexDir, 'index.html'), bodyStr)
            self.__parseHtml(cfg, indexDir, false)
              .then(({ html }) => {
                res.end(html)
              }).catch(err => {
                res.writeHead(500, JSON.stringify(err))
                res.end()
              })
          })
        } else {
          if (proxyRes.statusCode) {
            res = res.writeHead(proxyRes.statusCode, proxyRes.headers)
          }

          proxyRes.pipe(res)
        }
      })

      proxy.on('error', (error: Error, _: http.IncomingMessage, res: http.ServerResponse) => {
        console.error(error)
        res.writeHead(500, error.message)
      })

      const proxyServer = http.createServer((req, res) => {
        delete req.headers['accept-encoding']
        proxy.web(req, res)
      })
      proxyServer.on('upgrade', (req, socket, head) => {
        proxy.ws(req, socket, head)
      })

      const port = await findClosestOpenPort(parseInt(devUrl.port) + 1, devUrl.hostname)
      const devServer = proxyServer.listen(port)
      this.devServer = devServer
      devPath = `${devUrl.protocol}//localhost:${port}`
      cfg.build.devPath = devPath
      process.env.TAURI_CONFIG = JSON.stringify(cfg)
    } else {
      inlinedAssets = (await this.__parseHtml(cfg, devPath)).inlinedAssets
    }

    process.env.TAURI_INLINED_ASSETS = inlinedAssets.join('|')

    this.devPath = devPath

    const startDevTauri = async (): Promise<void> => {
      return await this.__runCargoCommand({
        cargoArgs: ['run'],
        dev: true,
        exitOnPanic: cfg.ctx.exitOnPanic
      })
    }

    // Start watching for tauri app changes
    // eslint-disable-next-line security/detect-non-literal-fs-filename

    let tauriPaths: string[] = []
    if (typeof cargoManifest.dependencies.tauri !== 'string' && cargoManifest.dependencies.tauri.path) {
      const tauriPath = path.resolve(tauriDir, cargoManifest.dependencies.tauri.path)
      tauriPaths = [
        tauriPath,
        `${tauriPath}-api`,
        `${tauriPath}-updater`,
        `${tauriPath}-utils`
      ]
    }

    if (!this.tauriWatcher) {
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
            ignored: [runningDevServer ? null : path.join(devPath, 'index.tauri.html')].concat(path.join(tauriDir, 'target'))
          }
        )
        .on(
          'change',
          debounce((changedPath: string) => {
            if (this.rewritingToml && changedPath.startsWith(path.join(tauriDir, 'Cargo.toml'))) {
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
          }, WATCHER_INTERVAL)
        )
    }

    return startDevTauri()
  }

  async build(cfg: TauriConfig): Promise<void> {
    if (cfg.build.beforeBuildCommand) {
      const [command, ...args] = cfg.build.beforeBuildCommand.split(' ')
      spawnSync(command, args, appDir)
    }

    const cargoManifest = this.__getManifest()
    this.__whitelistApi(cfg, cargoManifest as unknown as CargoManifest)
    this.__rewriteManifest(cargoManifest)

    const inlinedAssets = (await this.__parseHtml(cfg, cfg.build.distDir)).inlinedAssets
    process.env.TAURI_INLINED_ASSETS = inlinedAssets.join('|')

    const features = [
      cfg.tauri.embeddedServer.active ? 'embedded-server' : 'no-server'
    ]

    const buildFn = async (target?: string): Promise<void> =>
      await this.__runCargoCommand({
        cargoArgs: [
          cfg.tauri.bundle.active ? 'tauri-bundler' : 'build',
          '--features',
          ...features,
          ...(
            cfg.tauri.bundle.active && Array.isArray(cfg.tauri.bundle.targets) && cfg.tauri.bundle.targets.length
              ? ['--format'].concat(cfg.tauri.bundle.targets)
              : []
          )
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

  async __parseHtml(cfg: TauriConfig, indexDir: string, inlinerEnabled = cfg.tauri.inliner.active): Promise<{ inlinedAssets: string[], html: string }> {
    const inlinedAssets: string[] = []

    return await new Promise((resolve, reject) => {
      const indexPath = path.join(indexDir, 'index.html')
      if (!existsSync(indexPath)) {
        warn(
          `Error: cannot find index.html in "${indexDir}". Did you forget to build your web code or update the build.distDir in tauri.conf.json?`
        )
        reject(new Error('Could not find index.html in dist dir.'))
      }
      const originalHtml = readFileSync(indexPath).toString()

      const rewriteHtml = (html: string, interceptor?: (dom: JSDOM) => void): string => {
        const dom = new JSDOM(html)
        const document = dom.window.document
        if (interceptor !== undefined) {
          interceptor(dom)
        }

        // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
        if (!((cfg.ctx.dev && cfg.build.devPath.startsWith('http')) || cfg.tauri.embeddedServer.active)) {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-var-requires, @typescript-eslint/no-unsafe-member-access
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
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-var-requires, @typescript-eslint/no-unsafe-member-access
        tauriScript.text = require('../templates/tauri.js').default
        document.head.insertBefore(tauriScript, document.head.firstChild)

        if (cfg.build.withGlobalTauri) {
          const tauriUmdScript = document.createElement('script')
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-var-requires, @typescript-eslint/no-unsafe-member-access
          tauriUmdScript.text = require('../api/tauri.bundle.umd').default
          document.head.insertBefore(tauriUmdScript, document.head.firstChild)
        }

        const csp = cfg.tauri.security.csp
        if (csp) {
          const cspTag = document.createElement('meta')
          cspTag.setAttribute('http-equiv', 'Content-Security-Policy')
          cspTag.setAttribute('content', csp)
          document.head.appendChild(cspTag)
        }

        const newHtml = dom.serialize()
        writeFileSync(
          path.join(indexDir, 'index.tauri.html'),
          newHtml
        )
        return newHtml
      }

      const domInterceptor = cfg.tauri.embeddedServer.active ? undefined : (dom: JSDOM) => {
        const document = dom.window.document
        if (!cfg.ctx.dev) {
          document.querySelectorAll('link').forEach((link: HTMLLinkElement) => {
            link.removeAttribute('rel')
            link.removeAttribute('as')
          })
        }
      }

      if ((!cfg.ctx.dev && cfg.tauri.embeddedServer.active) || !inlinerEnabled) {
        const html = rewriteHtml(originalHtml, domInterceptor)
        resolve({ inlinedAssets, html })
      } else {
        const cwd = process.cwd()
        process.chdir(indexDir) // the inliner requires this to properly work
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call
        const inliner = new Inliner({ source: originalHtml }, (err: Error, html: string) => {
          process.chdir(cwd) // reset CWD
          if (err) {
            reject(err)
          } else {
            const rewrittenHtml = rewriteHtml(html, domInterceptor)
            resolve({ inlinedAssets, html: rewrittenHtml })
          }
        })
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
        inliner.on('progress', (event: string) => {
          const match = event.match(/([\S\d]+)\.([\S\d]+)/g)
          match && inlinedAssets.push(match[0])
        })
      }
    })
  }

  async stop(): Promise<void> {
    return await new Promise((resolve, reject) => {
      this.devServer?.close()
      this.tauriWatcher?.close().catch(reject)
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
    return await new Promise((resolve, reject) => {
      this.pid = spawn(
        'cargo',

        extraArgs ? cargoArgs.concat(['--']).concat(extraArgs) : cargoArgs,

        tauriDir,

        (code, pid) => {
          if (this.killPromise) {
            this.killPromise()
            this.killPromise = undefined
            resolve()
            return
          }

          if (pid !== this.pid) {
            resolve()
            return
          }

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

          if (dev) {
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
    if (!this.pid) {
      return await Promise.resolve()
    }

    log('Shutting down tauri process...')

    return await new Promise((resolve, reject) => {
      this.killPromise = resolve
      try {
        process.kill(this.pid)
        this.pid = 0
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
    const cargoManifest = toml.parse(tomlFile)
    return cargoManifest
  }

  __rewriteManifest(cargoManifest: JsonMap): void {
    const tomlPath = this.__getManifestPath()
    const output = toml.stringify(cargoManifest)

    this.rewritingToml = true
    writeFileSync(tomlPath, output)
    setTimeout(() => {
      this.rewritingToml = false
    }, WATCHER_INTERVAL * 2)
  }

  __whitelistApi(
    cfg: TauriConfig,
    manifest: CargoManifest
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

    if (cfg.tauri.cli) {
      tomlFeatures.push('cli')
    }

    if (typeof manifest.dependencies.tauri === 'string') {
      manifest.dependencies.tauri = {
        version: manifest.dependencies.tauri,
        features: tomlFeatures
      }
    } else {
      manifest.dependencies.tauri.features = tomlFeatures
    }
  }
}

export default Runner
