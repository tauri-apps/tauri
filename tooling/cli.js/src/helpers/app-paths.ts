// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join, resolve, isAbsolute, dirname } from 'path'
import logger from './logger'
import chalk from 'chalk'
import { createRequire } from 'module'

const warn = logger('tauri', chalk.red)

const require = createRequire(import.meta.url)
// eslint-disable-next-line @typescript-eslint/no-var-requires, @typescript-eslint/no-unsafe-assignment
const glob = require('glob')

function resolvePath(basePath: string, dir: string): string {
  return dir && isAbsolute(dir) ? dir : resolve(basePath, dir)
}

const getAppDir = (): string | null => {
  const dir = process.env.__TAURI_TEST_APP_DIR ?? process.cwd()
  // eslint-disable-next-line
  const matches: string[] = glob.sync(join(dir, `**/package.json`), {
    ignore: '**/node_modules/**'
  })

  if (matches.length === 0) {
    return null
  } else {
    return dirname(matches[0])
  }
}

const getTauriDir = (): string => {
  const dir = process.env.__TAURI_TEST_APP_DIR ?? process.cwd()
  // eslint-disable-next-line
  const matches: string[] = glob.sync(join(dir, `**/tauri.conf.json`))

  if (matches.length === 0) {
    warn(
      "Couldn't recognize the current folder as a part of a Tauri project. It must contain a `tauri.conf.json` file in any subfolder."
    )
    process.exit(1)
    return ''
  } else {
    return dirname(matches[0])
  }
}

const appDir = getAppDir() ?? resolve(getTauriDir(), '..')
const tauriDir = getTauriDir()

const resolveDir = {
  app: (dir: string) => resolvePath(appDir, dir),
  tauri: (dir: string) => resolvePath(tauriDir, dir)
}

export { appDir, tauriDir, resolveDir as resolve }
