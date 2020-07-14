import { ManagementType } from './types'
import { spawnSync } from '../../helpers/spawn'
import getScriptVersion from '../../helpers/get-script-version'
import logger from '../../helpers/logger'
import { createWriteStream, unlinkSync } from 'fs'
import https from 'https'

const log = logger('dependency:rust')

async function download(url: string, dest: string): Promise<void> {
  const file = createWriteStream(dest)
  return await new Promise((resolve, reject) => {
    https.get(url, response => {
      response.pipe(file)
      file.on('finish', function () {
        file.close()
        resolve()
      })
    }).on('error', function (err) {
      unlinkSync(dest)
      reject(err.message)
    })
  })
};

async function installRustup(): Promise<void> {
  return await download('https://sh.rustup.rs', 'rustup.sh')
    .then(() => {
      spawnSync('rustup.sh', [], process.cwd())
      unlinkSync('rustup.sh')
    })
}

async function manageDependencies(managementType: ManagementType): Promise<void> {
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

export {
  install,
  update
}
