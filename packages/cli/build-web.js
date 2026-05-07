// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const { existsSync } = require('node:fs')
const path = require('node:path')
const { spawnSync } = require('node:child_process')

const target = 'wasm32-wasip1-threads'

function command(name) {
  return process.platform === 'win32' ? `${name}.cmd` : name
}

function output(command, args) {
  const result = spawnSync(command, args, {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'inherit'],
  })

  if (result.status !== 0) {
    process.exit(result.status || 1)
  }

  return result.stdout.trim()
}

const sysroot = output('rustc', ['--print', 'sysroot'])
const crtReactor = path.join(
  sysroot,
  'lib',
  'rustlib',
  target,
  'lib',
  'self-contained',
  'crt1-reactor.o',
)

const env = { ...process.env, TARGET: 'web' }
if (!existsSync(crtReactor)) {
  console.error(`Missing ${crtReactor}. Install the ${target} Rust target.`)
  process.exit(1)
}

env.RUSTFLAGS = [
  env.RUSTFLAGS,
  `-C link-arg=${crtReactor}`,
  '-C link-arg=--export=_initialize',
]
  .filter(Boolean)
  .join(' ')

const args = [
  'build',
  '--platform',
  '--target',
  target,
  '--no-default-features',
  '--no-js',
  '--dts',
  'cli.wasi.d.ts',
  ...process.argv.slice(2),
]

const result = spawnSync(command('napi'), args, {
  env,
  stdio: 'inherit',
})

if (result.error) {
  console.error(result.error.message)
  process.exit(1)
}

process.exit(result.status ?? 1)
