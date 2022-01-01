// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { readFileSync, writeFileSync } from 'fs'
import { join } from 'path'

interface TauriConfJSON {
  build?: {
    beforeDevCommand?: string
    beforeBuildCommand?: string
    distDir?: string
    devPath?: string
    withGlobalTauri?: boolean
  }
}

export function updateTauriConf(
  f: (tauriConf: TauriConfJSON) => TauriConfJSON,
  cwd: string = process.cwd()
): void {
  const tauriConfPath = join(cwd, 'src-tauri', 'tauri.conf.json')
  const tauriConf = JSON.parse(
    readFileSync(tauriConfPath, 'utf8')
  ) as TauriConfJSON
  const output = f(tauriConf)
  writeFileSync(tauriConfPath, JSON.stringify(output, undefined, 2))
}
