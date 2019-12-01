const
  chokidar = require('chokidar')
const debounce = require('lodash.debounce')
const path = require('path')
const { readFileSync, writeFileSync } = require('fs-extra')

const
  { spawn } = require('./helpers/spawn')
const onShutdown = require('./helpers/on-shutdown')
const generator = require('./generator')
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

    generator.generate(cfg.tauri)

    this.devPath = devPath

    const args = ['--path', devPath.startsWith('http') ? devPath : path.resolve(appDir, devPath)]
    const features = ['dev']

    const startDevTauri = () => {
      return this.__runCargoCommand({
        cargoArgs: ['run', '--features', ...features],
        extraArgs: args
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
          this.run(require('./helpers/tauri-config')(cfg.ctx))
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

    generator.generate(cfg.tauri)

    const features = []
    if (cfg.tauri.embeddedServer.active) {
      features.push('embedded-server')
    }

    const buildFn = target => this.__runCargoCommand({
      cargoArgs: [cfg.tauri.bundle.active ? 'tauri-cli' : 'build']
        .concat(features.length ? ['--features', ...features] : [])
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

  stop () {
    return new Promise((resolve, reject) => {
      this.tauriWatcher && this.tauriWatcher.close()
      this.__stopCargo().then(resolve)
    })
  }

  __runCargoCommand ({
    cargoArgs,
    extraArgs
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
          } else if (cargoArgs.some(arg => arg === 'dev')) { // else it wasn't killed by us
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
