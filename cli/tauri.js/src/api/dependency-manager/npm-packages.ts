import { ManagementType, Result } from './types'
import {
  getNpmLatestVersion,
  getNpmPackageVersion,
  installNpmPackage,
  installNpmDevPackage,
  updateNpmPackage,
  semverLt
} from './util'
import logger from '../../helpers/logger'
import { resolve } from '../../helpers/app-paths'
import inquirer from 'inquirer'
import { existsSync } from 'fs'

const log = logger('dependency:npm-packages')

async function manageDependencies(
  managementType: ManagementType,
  dependencies: string[]
): Promise<Result> {
  const installedDeps = []
  const updatedDeps = []

  if (existsSync(resolve.app('package.json'))) {
    for (const dependency of dependencies) {
      const currentVersion = await getNpmPackageVersion(dependency)
      if (currentVersion === null) {
        log(`Installing ${dependency}...`)
        if (managementType === ManagementType.Install) {
          await installNpmPackage(dependency)
        } else if (managementType === ManagementType.InstallDev) {
          await installNpmDevPackage(dependency)
        }
        installedDeps.push(dependency)
      } else if (managementType === ManagementType.Update) {
        const latestVersion = await getNpmLatestVersion(dependency)
        if (semverLt(currentVersion, latestVersion)) {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-var-requires, @typescript-eslint/no-unsafe-member-access
          const inquired = await inquirer.prompt([
            {
              type: 'confirm',
              name: 'answer',
              message: `[NPM]: "${dependency}" latest version is ${latestVersion}. Do you want to update?`,
              default: false
            }
          ])
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-var-requires, @typescript-eslint/no-unsafe-member-access
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

const dependencies = ['tauri']

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
