// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { readFileSync, writeFileSync } from 'fs'
import { join } from 'path'

interface Package {
  name?: string
  scripts?: {}
}

export function updatePackageJson(appDirectory: string, appName: string): void {
  const pkgPath = join(appDirectory, 'package.json')
  const pkgString = readFileSync(pkgPath, 'utf8')
  const pkg = JSON.parse(pkgString) as Package
  const outputPkg = {
    ...pkg,
    name: appName,
    scripts: {
      ...pkg.scripts,
      tauri: 'tauri'
    }
  }
  writeFileSync(pkgPath, JSON.stringify(outputPkg, undefined, 2))
}
