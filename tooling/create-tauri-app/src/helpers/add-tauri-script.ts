// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { readFileSync, writeFileSync } from 'fs'
import { join } from 'path'

export function addTauriScript(appDirectory: string): void {
  const pkgPath = join(appDirectory, 'package.json')
  const pkgString = readFileSync(pkgPath, 'utf8')
  const pkg = JSON.parse(pkgString) as {
    scripts: {
      tauri: string
    }
  }

  const outputPkg = {
    ...pkg,
    scripts: {
      ...pkg.scripts,
      tauri: 'tauri'
    }
  }

  writeFileSync(pkgPath, JSON.stringify(outputPkg, undefined, 2))
}
