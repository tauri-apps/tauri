// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { resolve } from 'node:path'
import { spawnSync } from 'node:child_process'
import {
  existsSync,
  readFileSync,
  writeFileSync,
  rmSync,
  renameSync
} from 'node:fs'
import { beforeAll, describe, it } from 'vitest'

// Build CLI before tests, for local usage only.
// CI builds the CLI on different platforms and architectures
if (!process.env.CI) {
  beforeAll(() => {
    const cliDir = resolve(__dirname, '..')
    exec('pnpm', ['build:debug'], { cwd: cliDir })
  })
}

describe('[CLI] @tauri-apps/cli template', () => {
  it('init a project and builds it', { timeout: 15 * 60 * 1000 }, async () => {
    const cwd = process.cwd()
    const fixturePath = resolve(__dirname, './fixtures/empty')
    const tauriFixturePath = resolve(fixturePath, 'src-tauri')
    const outPath = resolve(tauriFixturePath, 'target')
    const cacheOutPath = resolve(fixturePath, 'target')

    process.chdir(fixturePath)

    const outExists = existsSync(outPath)
    if (outExists) {
      if (existsSync(cacheOutPath)) {
        rmSync(cacheOutPath, { recursive: true, force: true })
      }
      renameSync(outPath, cacheOutPath)
    }

    const cli = await import('../main.js')

    await cli.run([
      'init',
      '-vvv',
      '--directory',
      process.cwd(),
      '--force',
      '--tauri-path',
      resolve(__dirname, '../../..'),
      '--before-build-command',
      '',
      '--before-dev-command',
      '',
      '--ci'
    ])

    if (outExists) {
      renameSync(cacheOutPath, outPath)
    }

    process.chdir(tauriFixturePath)

    const manifestPath = resolve(tauriFixturePath, 'Cargo.toml')
    const manifestFile = readFileSync(manifestPath).toString()
    writeFileSync(manifestPath, `workspace = { }\n${manifestFile}`)

    const configPath = resolve(tauriFixturePath, 'tauri.conf.json')
    const config = readFileSync(configPath).toString()
    writeFileSync(configPath, config.replace('com.tauri.dev', 'com.tauri.test'))

    await cli.run(['build', '-vvv'])
    process.chdir(cwd)
  })
})

function exec(
  bin: string,
  args?: string[],
  opts?: {
    cwd?: string
  }
) {
  process.platform === 'win32'
    ? spawnSync('cmd', ['/c', bin, ...(args ?? [])], { cwd: opts?.cwd })
    : spawnSync(bin, args, { cwd: opts?.cwd })
}
