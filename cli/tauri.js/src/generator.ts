import { writeFileSync } from 'fs-extra'
import path from 'path'
import { tauriDir } from './helpers/app-paths'
import { TauriConfig } from './types/config'

export const generate = (tauriConfig: TauriConfig['tauri']): void => {
  const { bundle, ...cfg } = tauriConfig
  const outDir = tauriDir
  writeFileSync(path.join(outDir, 'config.json'), JSON.stringify(cfg))
  writeFileSync(path.join(outDir, 'bundle.json'), JSON.stringify(bundle))
}
