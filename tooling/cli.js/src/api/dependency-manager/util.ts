// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { spawnSync } from '../../helpers/spawn'
import { sync as crossSpawnSync } from 'cross-spawn'
import { appDir, resolve as appResolve } from '../../helpers/app-paths'
import { existsSync } from 'fs'
import semver from 'semver'

const useYarn = (): boolean =>
  process.env.npm_execpath
    ? process.env.npm_execpath.includes('yarn')
    : existsSync(appResolve.app('yarn.lock'))

function getCrateLatestVersion(crateName: string): string | null {
  const child = crossSpawnSync('cargo', ['search', crateName, '--limit', '1'])
  const output = String(child.output[1])
  // eslint-disable-next-line security/detect-non-literal-regexp
  const matches = new RegExp(crateName + ' = "(\\S+)"', 'g').exec(output)
  if (matches?.[1]) {
    return matches[1]
  } else {
    return null
  }
}

function getNpmLatestVersion(packageName: string): string {
  if (useYarn()) {
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

function getNpmPackageVersion(packageName: string): string | null {
  const child = useYarn()
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

function installNpmPackage(packageName: string): void {
  if (useYarn()) {
    spawnSync('yarn', ['add', packageName], appDir)
  } else {
    spawnSync('npm', ['install', packageName], appDir)
  }
}

function installNpmDevPackage(packageName: string): void {
  if (useYarn()) {
    spawnSync('yarn', ['add', packageName, '--dev'], appDir)
  } else {
    spawnSync('npm', ['install', packageName, '--save-dev'], appDir)
  }
}

function updateNpmPackage(packageName: string): void {
  if (useYarn()) {
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
