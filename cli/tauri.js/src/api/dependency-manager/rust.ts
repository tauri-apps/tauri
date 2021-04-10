// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { ManagementType } from './types'
import { spawnSync } from '../../helpers/spawn'
import getScriptVersion from '../../helpers/get-script-version'
import logger from '../../helpers/logger'
import { createWriteStream, unlinkSync } from 'fs'
import { resolve } from 'path'
import { platform } from 'os'
import https from 'https'

const log = logger('dependency:rust')

// eslint-disable-next-line @typescript-eslint/no-unused-vars
async function download(url: string, dest: string): Promise<void> {
  const file = createWriteStream(dest)
  return await new Promise((resolve, reject) => {
    https
      .get(url, (response) => {
        response.pipe(file)
        file.on('finish', function () {
          file.close()
          resolve()
        })
      })
      .on('error', function (err) {
        unlinkSync(dest)
        reject(err.message)
      })
  })
}

function installRustup(): void {
  if (platform() === 'win32') {
    return spawnSync(
      'powershell',
      [resolve(__dirname, '../../scripts/rustup-init.exe')],
      process.cwd()
    )
  }
  return spawnSync(
    '/bin/sh',
    [resolve(__dirname, '../../scripts/rustup-init.sh')],
    process.cwd()
  )
}

function manageDependencies(managementType: ManagementType): void {
  if (getScriptVersion('rustup') === null) {
    log('Installing rustup...')
    installRustup()
  }

  if (managementType === ManagementType.Update) {
    spawnSync('rustup', ['update'], process.cwd())
  }
}

function install(): void {
  return manageDependencies(ManagementType.Install)
}

function update(): void {
  return manageDependencies(ManagementType.Update)
}

export { install, update }
