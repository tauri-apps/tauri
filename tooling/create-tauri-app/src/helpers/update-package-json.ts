// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { readFileSync, writeFileSync } from 'fs'
import { join } from 'path'

interface PackageJSON {
  name?: string
  scripts?: {}
}

export function updatePackageJson(
  f: (pkg: PackageJSON) => PackageJSON,
  cwd: string = process.cwd()
): void {
  const pkgPath = join(cwd, 'package.json')
  const pkg = JSON.parse(readFileSync(pkgPath, 'utf8')) as PackageJSON
  const output = f(pkg)
  writeFileSync(pkgPath, JSON.stringify(output, undefined, 2))
}
