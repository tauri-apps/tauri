import { TauriConfig } from 'types'
import merge from 'webpack-merge'
import * as entry from '../entry'
import * as generator from '../generator'
import { tauriDir } from '../helpers/app-paths'
import getTauriConfig from '../helpers/tauri-config'
import Runner from '../runner'

export default async (config: TauriConfig): Promise<void> => {
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

  generator.generate(tauriConfig.tauri)
  entry.generate(tauriDir, tauriConfig)

  return tauri.run(tauriConfig)
}
