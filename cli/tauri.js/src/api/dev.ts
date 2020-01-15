import { TauriConfig } from 'types'
import merge from 'webpack-merge'
import Runner from '../runner'
const getTauriConfig = require('../helpers/tauri-config')

interface DevResult {
  promise: Promise<void>,
  runner: Runner
}

module.exports = (config: TauriConfig): DevResult => {
  const tauri = new Runner()
  const tauriConfig = getTauriConfig(
    merge(
      {
        ctx: {
          debug: true,
          dev: true
        }
      } as any,
      config as any
    ) as TauriConfig
  )

  return {
    runner: tauri,
    promise: tauri.run(tauriConfig)
  }
}
