import { ensureDirSync, writeFileSync } from 'fs-extra'
import { template } from 'lodash'
import path from 'path'
import { TauriConfig } from './types/config'

export const generate = (outDir: string, cfg: TauriConfig): void => {
  // this MUST be from the templates repo
  const apiTemplate = require('../templates/tauri.js').default
  const compiledApi = template(apiTemplate)

  ensureDirSync(outDir)
  writeFileSync(path.join(outDir, 'tauri.js'), compiledApi(cfg), 'utf-8')
}
