// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { ManagementType } from './types'
import { spawnSync } from '../../helpers/spawn'
import getScriptVersion from '../../helpers/get-script-version'
import { downloadRustup } from '../../helpers/download-binary'
import logger from '../../helpers/logger'
import { createWriteStream, unlinkSync, existsSync } from 'fs'
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

async function installRustup(): Promise<void> {
  const assetName =
    platform() === 'win32' ? 'rustup-init.exe' : 'rustup-init.sh'
  const rustupPath = resolve(__dirname, `../../bin/${assetName}`)
  if (!existsSync(rustupPath)) {
    await downloadRustup()
  }
  if (platform() === 'win32') {
    return spawnSync('powershell', ['-NoProfile', rustupPath], process.cwd())
  }
  return spawnSync('/bin/sh', [rustupPath], process.cwd())
}

async function manageDependencies(
  managementType: ManagementType
): Promise<void> {
  if (getScriptVersion('rustup') === null) {
    log('Installing rustup...')
    await installRustup()
  }

  if (managementType === ManagementType.Update) {
    spawnSync('rustup', ['update'], process.cwd())
  }
}

async function install(): Promise<void> {
  return await manageDependencies(ManagementType.Install)
}

async function update(): Promise<void> {
  return await manageDependencies(ManagementType.Update)
}

export { install, update }
