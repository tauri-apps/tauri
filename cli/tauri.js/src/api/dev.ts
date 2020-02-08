import { TauriConfig } from 'types'
import merge from 'webpack-merge'
import Runner from '../runner'
import getTauriConfig from '../helpers/tauri-config'

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
