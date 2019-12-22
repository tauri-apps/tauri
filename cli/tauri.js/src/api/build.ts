import { TauriConfig } from 'types'
import merge from 'webpack-merge'
import * as entry from '../entry'
import * as generator from '../generator'
import { tauriDir } from '../helpers/app-paths'
import getTauriConfig from '../helpers/tauri-config'
import Runner from '../runner'

module.exports = async (config: TauriConfig): Promise<void> => {
  // TODO: why is this being passed?
  // @ts-ignore
  const tauri = new Runner({ modeDir: tauriDir })
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

  generator.generate(tauriConfig.tauri)
  entry.generate(tauriDir, tauriConfig)

  return tauri.build(tauriConfig)
}
