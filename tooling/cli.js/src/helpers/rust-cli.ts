// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { existsSync } from 'fs'
import { resolve, join } from 'path'
import { spawnSync, spawn } from './spawn'
import { CargoManifest } from '../types/cargo'
import { downloadCli } from './download-cli'

const currentTauriCliVersion = (): string => {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-var-requires, @typescript-eslint/no-unsafe-member-access
  const tauriCliManifest = require('../../../cli.rs/Cargo.toml') as CargoManifest
  return tauriCliManifest.package.version
}

export async function runOnRustCli(
  command: string,
  args: string[]
): Promise<{ pid: number; promise: Promise<void> }> {
  const targetPath = resolve(__dirname, '../..')
  const targetCliPath = join(
    targetPath,
    'bin/tauri-cli' + (process.platform === 'win32' ? '.exe' : '')
  )

  let resolveCb: () => void
  let rejectCb: () => void
  let pid: number
  const promise = new Promise<void>((resolve, reject) => {
    resolveCb = resolve
    rejectCb = () => reject(new Error())
  })
  const onClose = (code: number, pid: number): void => {
    if (code === 0) {
      resolveCb()
    } else {
      rejectCb()
    }
  }

  if (existsSync(targetCliPath)) {
    pid = spawn(
      targetCliPath,
      ['tauri', command, ...args],
      process.cwd(),
      onClose
    )
  } else if (process.env.NODE_ENV === 'production') {
    await downloadCli()
    pid = spawn(
      targetCliPath,
      ['tauri', command, ...args],
      process.cwd(),
      onClose
    )
  } else {
    if (existsSync(resolve(targetPath, '../bundler'))) {
      // running local CLI
      const cliPath = resolve(targetPath, '../cli.rs')
      spawnSync('cargo', ['build', '--release'], cliPath)
      const localCliPath = resolve(
        targetPath,
        '../cli.rs/target/release/cargo-tauri'
      )
      pid = spawn(
        localCliPath,
        ['tauri', command, ...args],
        process.cwd(),
        onClose
      )
    } else {
      spawnSync(
        'cargo',
        [
          'install',
          '--root',
          targetPath,
          'tauri-cli',
          '--version',
          currentTauriCliVersion()
        ],
        process.cwd()
      )
      pid = spawn(
        targetCliPath,
        ['tauri', command, ...args],
        process.cwd(),
        onClose
      )
    }
  }

  return { pid, promise }
}
