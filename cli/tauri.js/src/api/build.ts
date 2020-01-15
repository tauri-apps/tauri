import { TauriConfig } from 'types'
import merge from 'webpack-merge'
import Runner from '../runner'
const getTauriConfig = require('../helpers/tauri-config')

interface BuildResult {
  promise: Promise<void>
  runner: Runner
}

module.exports = (config: TauriConfig): BuildResult => {
  const tauri = new Runner()
  const tauriConfig = getTauriConfig(
    merge(
      {
        ctx: {
          prod: true
        }
      } as any,
      config as any
    ) as TauriConfig
  )

  return {
    runner: tauri,
    promise: tauri.build(tauriConfig)
  }
}
