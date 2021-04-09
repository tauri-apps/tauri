// SPDX-License-Identifier: Apache-2.0 OR MIT

import { spawnSync } from './../../helpers/spawn'
import {
  CargoManifest,
  CargoManifestDependency,
  CargoLock
} from './../../types/cargo'
import { ManagementType, Result } from './types'
import { getCrateLatestVersion, semverLt } from './util'
import logger from '../../helpers/logger'
import { resolve as appResolve, tauriDir } from '../../helpers/app-paths'
import { readFileSync, writeFileSync, existsSync } from 'fs'
import toml from '@tauri-apps/toml'
import inquirer from 'inquirer'

const log = logger('dependency:crates')

const dependencies = ['tauri']

function readToml<T>(tomlPath: string): T | null {
  if (existsSync(tomlPath)) {
    const manifest = readFileSync(tomlPath).toString()
    return (toml.parse(manifest) as any) as T
  }
  return null
}

function dependencyDefinition(version: string): CargoManifestDependency {
  return { version: version.substring(0, version.lastIndexOf('.')) }
}

async function manageDependencies(
  managementType: ManagementType
): Promise<Result> {
  const installedDeps = []
  const updatedDeps = []
  const result: Result = new Map<ManagementType, string[]>()

  const manifest = readToml<CargoManifest>(appResolve.tauri('Cargo.toml'))

  if (manifest === null) {
    log('Cargo.toml not found. Skipping crates check...')
    return result
  }

  const lockPath = appResolve.tauri('Cargo.lock')
  if (!existsSync(lockPath)) {
    spawnSync('cargo', ['generate-lockfile'], tauriDir)
  }
  const lock = readToml<CargoLock>(lockPath)

  for (const dependency of dependencies) {
    const lockPackages = lock
      ? lock.package.filter((pkg) => pkg.name === dependency)
      : []
    // eslint-disable-next-line security/detect-object-injection
    const manifestDep = manifest.dependencies[dependency]
    const currentVersion =
      lockPackages.length === 1
        ? lockPackages[0].version
        : typeof manifestDep === 'string'
        ? manifestDep
        : manifestDep?.version
    if (currentVersion === undefined) {
      log(`Installing ${dependency}...`)
      const latestVersion = await getCrateLatestVersion(dependency)
      // eslint-disable-next-line security/detect-object-injection
      manifest.dependencies[dependency] = dependencyDefinition(latestVersion)
      installedDeps.push(dependency)
    } else if (managementType === ManagementType.Update) {
      const latestVersion = await getCrateLatestVersion(dependency)
      if (semverLt(currentVersion, latestVersion)) {
        const inquired = (await inquirer.prompt([
          {
            type: 'confirm',
            name: 'answer',
            message: `[CRATES] "${dependency}" latest version is ${latestVersion}. Do you want to update?`,
            default: false
          }
        ])) as { answer: boolean }
        if (inquired.answer) {
          log(`Updating ${dependency}...`)
          // eslint-disable-next-line security/detect-object-injection
          manifest.dependencies[dependency] = dependencyDefinition(
            latestVersion
          )
          updatedDeps.push(dependency)
        }
      } else {
        log(`"${dependency}" is up to date`)
      }
    } else {
      log(`"${dependency}" is already installed`)
    }
  }

  if (installedDeps.length || updatedDeps.length) {
    writeFileSync(
      appResolve.tauri('Cargo.toml'),
      toml.stringify(manifest as any)
    )
  }
  if (updatedDeps.length) {
    if (!existsSync(appResolve.tauri('Cargo.lock'))) {
      spawnSync('cargo', ['generate-lockfile'], tauriDir)
    }
    spawnSync(
      'cargo',
      [
        'update',
        '--aggressive',
        ...updatedDeps.reduce<string[]>(
          (initialValue, dep) => [...initialValue, '-p', dep],
          []
        )
      ],
      tauriDir
    )
  }

  result.set(ManagementType.Install, installedDeps)
  result.set(ManagementType.Update, updatedDeps)

  return result
}

async function install(): Promise<Result> {
  return await manageDependencies(ManagementType.Install)
}

async function update(): Promise<Result> {
  return await manageDependencies(ManagementType.Update)
}

export { install, update }
