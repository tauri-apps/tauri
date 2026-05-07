// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const { copyFileSync, existsSync } = require('node:fs')
const path = require('node:path')
const { spawnSync } = require('node:child_process')

const webArtifactsDir = path.join(
  'artifacts',
  'bindings-wasm32-wasip1-threads',
)
const wasmPackageDir = path.join('npm', 'wasm32-wasi')

function command(name) {
  return process.platform === 'win32' ? `${name}.cmd` : name
}

function copyRequiredFile(from, to) {
  if (!existsSync(from)) {
    console.error(`Missing artifact: ${from}`)
    process.exit(1)
  }

  copyFileSync(from, to)
}

const result = spawnSync(
  command('napi'),
  ['artifacts', '--build-output-dir', webArtifactsDir],
  {
    stdio: 'inherit',
  },
)

if (result.error) {
  console.error(result.error.message)
  process.exit(1)
}

if (result.status !== 0) {
  process.exit(result.status ?? 1)
}

for (const file of [
  'cli.wasi-browser.js',
  'cli.wasi.cjs',
  'cli.wasi.d.ts',
  'cli.wasm32-wasi.debug.wasm',
  'cli.wasm32-wasi.wasm',
  'wasi-worker-browser.mjs',
  'wasi-worker.mjs',
]) {
  copyRequiredFile(path.join(webArtifactsDir, file), file)
}

for (const file of ['cli.wasi.d.ts', 'cli.wasm32-wasi.debug.wasm']) {
  copyRequiredFile(
    path.join(webArtifactsDir, file),
    path.join(wasmPackageDir, file),
  )
}
