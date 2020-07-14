import https from 'https'
import { IncomingMessage } from 'http'
import { spawnSync } from '../../helpers/spawn'
import { sync as crossSpawnSync } from 'cross-spawn'
import { appDir, resolve as appResolve } from '../../helpers/app-paths'
import { existsSync } from 'fs'

const BASE_URL = 'https://docs.rs/crate/'

async function getCrateLatestVersion(crateName: string): Promise<string> {
  return await new Promise((resolve, reject) => {
    const url = `${BASE_URL}${crateName}`
    https.get(url, (res: IncomingMessage) => {
      if (res.statusCode !== 302 || !res.headers.location) {
        reject(res)
      } else {
        const version = res.headers.location.replace(url + '/', '')
        resolve(version)
      }
    })
  })
}

async function getNpmLatestVersion(packageName: string): Promise<string> {
  return await new Promise((resolve, reject) => {
    resolve('1.0.0')
  })
}

async function getNpmPackageVersion(packageName: string): Promise<string> {
  return await new Promise((resolve, reject) => {
    const child = crossSpawnSync('npm', ['list', packageName, 'version', '--depth', '0'], { cwd: appDir })
    if (child.status === 0) {
      const output = String(child.output[1])
      const matches = new RegExp(packageName + '@(\\S+)', 'g').exec(output)
      if (matches && matches[1]) {
        resolve(matches[1])
      } else {
        reject(`Failed to get ${packageName} version`)
      }
    } else {
      reject(child.output)
    }
  })
}

function installNpmPackage(packageName: string) {
  const usesYarn = existsSync(appResolve.app('yarn.lock'))
  if (usesYarn) {
    spawnSync('yarn', ['add', packageName], appDir)
  } else {
    spawnSync('npm', ['install', packageName], appDir)
  }
}

function updateNpmPackage(packageName: string) {
  const usesYarn = existsSync(appResolve.app('yarn.lock'))
  if (usesYarn) {
    spawnSync('yarn', ['upgrade', packageName, '--latest'], appDir)
  } else {
    spawnSync('npm', ['install', `${packageName}@latest`], appDir)
  }
}

export {
  getCrateLatestVersion,
  getNpmLatestVersion,
  getNpmPackageVersion,
  installNpmPackage,
  updateNpmPackage
}
