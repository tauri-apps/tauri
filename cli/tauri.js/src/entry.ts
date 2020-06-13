import { ensureDirSync, writeFileSync } from 'fs-extra'
import { template } from 'lodash'
import path from 'path'
import { TauriConfig } from './types/config'

export const generate = (outDir: string, cfg: TauriConfig): void => {
  // this MUST be from the templates repo
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-var-requires
  const apiTemplate = require('../templates/tauri.js').default
  const compiledApi = template(apiTemplate)

  ensureDirSync(outDir)
  writeFileSync(path.join(outDir, 'tauri.js'), compiledApi(cfg), 'utf-8')
}
