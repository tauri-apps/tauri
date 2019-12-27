import { TauriConfig } from 'types'
import merge from 'webpack-merge'
const getTauriConfig = require('../helpers/tauri-config')
import Runner from '../runner'

module.exports = async (config: TauriConfig): Promise<void> => {
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

  return tauri.run(tauriConfig)
}
