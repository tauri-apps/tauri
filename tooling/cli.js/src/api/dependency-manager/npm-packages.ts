// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { Answer, ManagementType, Result } from './types'
import {
  getNpmLatestVersion,
  getNpmPackageVersion,
  installNpmPackage,
  installNpmDevPackage,
  updateNpmPackage,
  semverLt,
  getManager
} from './util'
import logger from '../../helpers/logger'
import { resolve } from '../../helpers/app-paths'
import inquirer from 'inquirer'
import { existsSync } from 'fs'
import { sync as crossSpawnSync } from 'cross-spawn'

const log = logger('dependency:npm-packages')

async function manageDependencies(
  managementType: ManagementType,
  dependencies: string[]
): Promise<Result> {
  const installedDeps = []
  const updatedDeps = []

  const npmChild = crossSpawnSync('npm', ['--version'])
  const yarnChild = crossSpawnSync('yarn', ['--version'])
  const pnpmChild = crossSpawnSync('pnpm', ['--version'])
  if (
    (npmChild.status ?? npmChild.error) &&
    (yarnChild.status ?? yarnChild.error) &&
    (pnpmChild.status ?? pnpmChild.error)
  ) {
    throw new Error(
      'must have installed one of the following package managers `npm`, `yarn`, `pnpm` to manage dependenices'
    )
  }

  if (existsSync(resolve.app('package.json'))) {
    for (const dependency of dependencies) {
      const currentVersion = getNpmPackageVersion(dependency)
      const packageManager = getManager().type.toUpperCase()

      if (currentVersion === null) {
        log(`Installing ${dependency}...`)
        if (
          managementType === ManagementType.Install ||
          managementType === ManagementType.InstallDev
        ) {
          const prefix =
            managementType === ManagementType.InstallDev
              ? ' as dev-dependency'
              : ''

          const inquired = await inquirer.prompt<Answer>([
            {
              type: 'confirm',
              name: 'answer',
              message: `[${packageManager}]: "Do you want to install ${dependency}${prefix}?"`,
              default: false
            }
          ])

          if (inquired.answer) {
            if (managementType === ManagementType.Install) {
              installNpmPackage(dependency)
            } else if (managementType === ManagementType.InstallDev) {
              installNpmDevPackage(dependency)
            }
            installedDeps.push(dependency)
          }
        }
      } else if (managementType === ManagementType.Update) {
        const latestVersion = getNpmLatestVersion(dependency)

        if (semverLt(currentVersion, latestVersion)) {
          const inquired = await inquirer.prompt<Answer>([
            {
              type: 'confirm',
              name: 'answer',
              message: `[${packageManager}]: "${dependency}" latest version is ${latestVersion}. Do you want to update?`,
              default: false
            }
          ])

          if (inquired.answer) {
            log(`Updating ${dependency}...`)
            updateNpmPackage(dependency)
            updatedDeps.push(dependency)
          }
        } else {
          log(`"${dependency}" is up to date`)
        }
      } else {
        log(`"${dependency}" is already installed`)
      }
    }
  }

  const result: Result = new Map<ManagementType, string[]>()
  result.set(ManagementType.Install, installedDeps)
  result.set(ManagementType.Update, updatedDeps)

  return result
}

const dependencies = ['@tauri-apps/api', '@tauri-apps/cli']

async function install(): Promise<Result> {
  return await manageDependencies(ManagementType.Install, dependencies)
}

async function installThese(dependencies: string[]): Promise<Result> {
  return await manageDependencies(ManagementType.Install, dependencies)
}

async function installTheseDev(dependencies: string[]): Promise<Result> {
  return await manageDependencies(ManagementType.InstallDev, dependencies)
}

async function update(): Promise<Result> {
  return await manageDependencies(ManagementType.Update, dependencies)
}

export { install, installThese, installTheseDev, update }
