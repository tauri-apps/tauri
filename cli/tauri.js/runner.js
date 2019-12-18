const
  chokidar = require('chokidar')
const debounce = require('lodash.debounce')
const path = require('path')
const { readFileSync, writeFileSync } = require('fs-extra')

const
  { spawn } = require('./helpers/spawn')
const onShutdown = require('./helpers/on-shutdown')
const generator = require('./generator')
const entry = require('./entry')
const { appDir, tauriDir } = require('./helpers/app-paths')

const logger = require('./helpers/logger')
const log = logger('app:tauri', 'green')
const warn = logger('app:tauri (template)', 'red')

class Runner {
  constructor () {
    this.pid = 0
    this.tauriWatcher = null
    onShutdown(() => {
      this.stop()
    })
  }

  async run (cfg) {
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

    const runningDevServer = devPath.startsWith('http')
    let inlinedAssets = []

    if (!runningDevServer) {
      inlinedAssets = await this.__parseHtml(path.resolve(appDir, devPath))
    }

    generator.generate({
      devPath: runningDevServer ? devPath : path.resolve(appDir, devPath),
      inlinedAssets,
      ...cfg.tauri
    })
    entry.generate(tauriDir, cfg)

    this.devPath = devPath

    const features = runningDevServer ? ['dev-server'] : []

    const startDevTauri = () => {
      return this.__runCargoCommand({
        cargoArgs: ['run'].concat(features.length ? ['--features', ...features] : []),
        dev: true
      })
    }

    // Start watching for tauri app changes
    this.tauriWatcher = chokidar
      .watch([
        path.join(tauriDir, 'src'),
        path.join(tauriDir, 'Cargo.toml'),
        path.join(tauriDir, 'build.rs'),
        path.join(appDir, 'tauri.conf.js')
      ], {
        watchers: {
          chokidar: {
            ignoreInitial: true
          }
        }
      })
      .on('change', debounce(async (path) => {
        await this.__stopCargo()
        if (path.includes('tauri.conf.js')) {
          this.run(require('./helpers/tauri-config')({ ctx: cfg.ctx }))
        } else {
          startDevTauri()
        }
      }, 1000))

    return startDevTauri()
  }

  async build (cfg) {
    this.__manipulateToml(toml => {
      this.__whitelistApi(cfg, toml)
    })

    const inlinedAssets = await this.__parseHtml(cfg.build.distDir)

    generator.generate({
      inlinedAssets,
      ...cfg.tauri
    })
    entry.generate(tauriDir, cfg)
    

    const features = [
      cfg.tauri.embeddedServer.active ? 'embedded-server' : 'no-server'
    ]

    const buildFn = target => this.__runCargoCommand({
      cargoArgs: [cfg.tauri.bundle.active ? 'tauri-cli' : 'build', '--features', ...features]
        .concat(cfg.ctx.debug ? [] : ['--release'])
        .concat(target ? ['--target', target] : [])
    })

    if (cfg.ctx.debug || !cfg.ctx.targetName) {
      // on debug mode or if no target specified,
      // build only for the current platform
      await buildFn()
    } else {
      const targets = cfg.ctx.target.split(',')

      for (const target of targets) {
        await buildFn(target)
      }
    }
  }

  __parseHtml (indexDir) {
    const Inliner = require('@tauri-apps/tauri-inliner')
    const jsdom = require('jsdom')
    const { JSDOM } = jsdom
    const inlinedAssets = []
    
    return new Promise((resolve, reject) => {
      new Inliner(path.join(indexDir, 'index.html'), (err, html) => {
        if (err) {
          reject(err)
        } else {
          const dom = new JSDOM(html)
          const document = dom.window.document
          document.querySelectorAll('link').forEach(link => {
            link.removeAttribute('rel')
            link.removeAttribute('as')
          })

          const tauriScript = document.createElement('script')
          tauriScript.text = readFileSync(path.join(tauriDir, 'tauri.js'))
          document.body.insertBefore(tauriScript, document.body.firstChild)
          
          writeFileSync(path.join(indexDir, 'index.tauri.html'), dom.serialize())
          resolve(inlinedAssets)
        }
      }).on('progress', event => {
        const match = event.match(/([\S\d]+)\.([\S\d]+)/g)
        match && inlinedAssets.push(match[0])
      })
    })
  }

  stop () {
    return new Promise((resolve, reject) => {
      this.tauriWatcher && this.tauriWatcher.close()
      this.__stopCargo().then(resolve)
    })
  }

  __runCargoCommand ({
    cargoArgs,
    extraArgs,
    dev = false
  }) {
    return new Promise(resolve => {
      this.pid = spawn(
        'cargo',

        extraArgs
          ? cargoArgs.concat(['--']).concat(extraArgs)
          : cargoArgs,

        tauriDir,

        code => {
          if (code) {
            warn()
            warn('⚠️  [FAIL] Cargo CLI has failed')
            warn()
            process.exit(1)
          }

          if (this.killPromise) {
            this.killPromise()
            this.killPromise = null
          } else if (dev) {
            warn()
            warn('Cargo process was killed. Exiting...')
            warn()
            process.exit(0)
          }
        }
      )

      resolve()
    })
  }

  __stopCargo () {
    const pid = this.pid

    if (!pid) {
      return Promise.resolve()
    }

    log('Shutting down tauri process...')
    this.pid = 0

    return new Promise((resolve, reject) => {
      this.killPromise = resolve
      process.kill(pid)
    })
  }

  __manipulateToml (callback) {
    const toml = require('@tauri-apps/toml')
    const tomlPath = path.join(tauriDir, 'Cargo.toml')
    const tomlFile = readFileSync(tomlPath)
    const tomlContents = toml.parse(tomlFile)

    callback(tomlContents)

    const output = toml.stringify(tomlContents)
    writeFileSync(tomlPath, output)
  }

  __whitelistApi (cfg, tomlContents) {
    const tomlFeatures = []

    if (cfg.tauri.whitelist.all) {
      tomlFeatures.push('all-api')
    } else {
      const whitelist = Object.keys(cfg.tauri.whitelist).filter(w => cfg.tauri.whitelist[w] === true)
      tomlFeatures.push(...whitelist)
    }

    if (cfg.tauri.edge.active) {
      tomlFeatures.push('edge')
    }

    tomlContents.dependencies.tauri.features = tomlFeatures
  }
}

module.exports = Runner
