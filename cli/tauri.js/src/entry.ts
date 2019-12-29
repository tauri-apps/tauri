import { ensureDirSync, writeFileSync } from 'fs-extra'
import  { template } from 'lodash'
import path from 'path'
import { TauriConfig } from './types/config'

export const generate = (outDir: string, cfg: TauriConfig): void => {
  // this MUST be from the templates repo
  const apiTemplate = require('../templates/tauri.js').default
  const apiContent = template(apiTemplate)

  ensureDirSync(outDir)
  writeFileSync(path.join(outDir, 'tauri.js'), apiContent(cfg), 'utf-8')
}
