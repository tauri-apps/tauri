// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import https from 'https'
import { IncomingMessage } from 'http'
import { spawnSync } from '../../helpers/spawn'
import { sync as crossSpawnSync } from 'cross-spawn'
import { appDir, resolve as appResolve } from '../../helpers/app-paths'
import { existsSync } from 'fs'
import semver from 'semver'

const BASE_URL = 'https://docs.rs/crate/'

async function useYarn(): Promise<boolean> {
  const hasYarnLockfile = existsSync(appResolve.app('yarn.lock'))
  if (hasYarnLockfile) {
    return true
  } else {
    return await new Promise((resolve) => {
      const child = crossSpawnSync('npm', ['--version'])
      resolve(!!(child.status ?? child.error))
    })
  }
}

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
  if (await useYarn()) {
    const child = crossSpawnSync(
      'yarn',
      ['info', packageName, 'versions', '--json'],
      {
        cwd: appDir
      }
    )
    const output = String(child.output[1])
    const packageJson = JSON.parse(output) as { data: string[] }
    return packageJson.data[packageJson.data.length - 1]
  } else {
    const child = crossSpawnSync('npm', ['show', packageName, 'version'], {
      cwd: appDir
    })
    return String(child.output[1]).replace('\n', '')
  }
}

async function getNpmPackageVersion(
  packageName: string
): Promise<string | null> {
  const child = (await useYarn())
    ? crossSpawnSync(
        'yarn',
        ['list', '--pattern', packageName, '--depth', '0'],
        {
          cwd: appDir
        }
      )
    : crossSpawnSync('npm', ['list', packageName, 'version', '--depth', '0'], {
        cwd: appDir
      })
  const output = String(child.output[1])
  // eslint-disable-next-line security/detect-non-literal-regexp
  const matches = new RegExp(packageName + '@(\\S+)', 'g').exec(output)
  if (matches?.[1]) {
    return matches[1]
  } else {
    return null
  }
}

async function installNpmPackage(packageName: string): Promise<void> {
  if (await useYarn()) {
    spawnSync('yarn', ['add', packageName], appDir)
  } else {
    spawnSync('npm', ['install', packageName], appDir)
  }
}

async function installNpmDevPackage(packageName: string): Promise<void> {
  if (await useYarn()) {
    spawnSync('yarn', ['add', packageName, '--dev'], appDir)
  } else {
    spawnSync('npm', ['install', packageName, '--save-dev'], appDir)
  }
}

function updateNpmPackage(packageName: string): void {
  const usesYarn = existsSync(appResolve.app('yarn.lock'))
  if (usesYarn) {
    spawnSync('yarn', ['upgrade', packageName, '--latest'], appDir)
  } else {
    spawnSync('npm', ['install', `${packageName}@latest`], appDir)
  }
}

function padVersion(version: string): string {
  let count = (version.match(/\./g) ?? []).length
  while (count < 2) {
    count++
    version += '.0'
  }
  return version
}

function semverLt(first: string, second: string): boolean {
  return semver.lt(padVersion(first), padVersion(second))
}

export {
  useYarn,
  getCrateLatestVersion,
  getNpmLatestVersion,
  getNpmPackageVersion,
  installNpmPackage,
  installNpmDevPackage,
  updateNpmPackage,
  padVersion,
  semverLt
}
