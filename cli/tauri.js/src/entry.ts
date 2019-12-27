import { ensureDirSync, writeFileSync } from 'fs-extra'
import compileTemplate from 'lodash.template'
import path from 'path'
import { TauriConfig } from './types/config'

export const generate = (outDir: string, cfg: TauriConfig): void => {
  // this MUST be from the templates repo
  const apiTemplate = require('../templates/tauri.js').default
  const apiContent = compileTemplate(apiTemplate)(cfg)

  ensureDirSync(outDir)
  writeFileSync(path.join(outDir, 'tauri.js'), apiContent, 'utf-8')
}
