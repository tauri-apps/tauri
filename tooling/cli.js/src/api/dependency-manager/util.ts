// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { sync as crossSpawnSync } from 'cross-spawn'
import { resolve as appResolve } from '../../helpers/app-paths'
import { existsSync } from 'fs'
import semver from 'semver'
import { IManager, NpmManager, YarnManager, PnpmManager } from './managers'

const getManager = (): IManager => {
  if (existsSync(appResolve.app('yarn.lock'))) {
    return new YarnManager()
  } else if (existsSync(appResolve.app('pnpm-lock.yaml'))) {
    return new PnpmManager()
  } else {
    return new NpmManager()
  }
}

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
  return getManager().getLatestVersion(packageName)
}

function getNpmPackageVersion(packageName: string): string | null {
  return getManager().getPackageVersion(packageName)
}

function installNpmPackage(packageName: string): void {
  return getManager().installPackage(packageName)
}

function installNpmDevPackage(packageName: string): void {
  return getManager().installDevPackage(packageName)
}

function updateNpmPackage(packageName: string): void {
  return getManager().updatePackage(packageName)
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
  return first !== second
  // When version 1.0.0 is released this code should work again
  // return semver.lt(padVersion(first), padVersion(second))
}

export {
  getManager,
  getCrateLatestVersion,
  getNpmLatestVersion,
  getNpmPackageVersion,
  installNpmPackage,
  installNpmDevPackage,
  updateNpmPackage,
  padVersion,
  semverLt
}
