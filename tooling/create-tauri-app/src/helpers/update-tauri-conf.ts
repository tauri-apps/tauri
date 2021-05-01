// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { readFileSync, writeFileSync } from 'fs'
import { join } from 'path'
import { TauriBuildConfig } from '../types/config'

export function updateTauriConf(
  appDirectory: string,
  cfg: TauriBuildConfig
): void {
  const tauriConfPath = join(appDirectory, 'src-tauri', 'tauri.conf.json')
  const tauriConfString = readFileSync(tauriConfPath, 'utf8')
  const tauriConf = JSON.parse(tauriConfString) as {
    build: TauriBuildConfig
  }

  const outputPkg: { build: TauriBuildConfig } = {
    ...tauriConf,
    build: {
      ...tauriConf.build,
      beforeBuildCommand: cfg.beforeBuildCommand,
      beforeDevCommand: cfg.beforeDevCommand
    }
  }

  writeFileSync(tauriConfPath, JSON.stringify(outputPkg, undefined, 2))
}
