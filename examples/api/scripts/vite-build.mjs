// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { spawn } from 'node:child_process'
import { dirname, join } from 'node:path'
import { createRequire } from 'node:module'

if (process.platform === 'freebsd' && process.arch === 'arm64') {
  console.log(
    'Skipping Vite build on FreeBSD arm64 because Rolldown does not publish a native binding for this platform.'
  )
  process.exit(0)
}

const require = createRequire(import.meta.url)
const vitePackageJson = require.resolve('vite/package.json')
const viteBin = join(dirname(vitePackageJson), 'bin', 'vite.js')

const child = spawn(process.execPath, [viteBin, 'build'], {
  stdio: 'inherit'
})

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal)
  } else {
    process.exit(code ?? 1)
  }
})
