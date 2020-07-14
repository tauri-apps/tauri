import { ManagementType } from './types'
import { getNpmLatestVersion, getNpmPackageVersion, installNpmPackage, updateNpmPackage } from './util'
import logger from '../../helpers/logger'
import semver from 'semver'
import inquirer from 'inquirer'

const log = logger('dependency:npm-packages')

const dependencies = ['tauri']

async function manageDependencies(managementType: ManagementType) {
  for (const dependency of dependencies) {
    const currentVersion = await getNpmPackageVersion(dependency)
    if (currentVersion === null) {
      log(`Installing ${dependency}...`)
      installNpmPackage(dependency)
    } else if (managementType === ManagementType.Update) {
      const latestVersion = await getNpmLatestVersion(dependency)
      if (semver.lt(currentVersion, latestVersion)) {
        const { answer } = await inquirer.prompt([{
          type: 'confirm',
          name: 'answer',
          message: `${dependency} latest version is ${latestVersion}. Do you want to update?`,
          default: false
        }])
        if (answer) {
          log(`Updating ${dependency}...`)
          updateNpmPackage(dependency)
        }
      } else {
        log(`${dependency} is up to date`)
      }
    } else {
      log(`${dependency} is already installed`)
    }
  }
}

async function install() {
  return await manageDependencies(ManagementType.Install)
}

async function update() {
  return await manageDependencies(ManagementType.Update)
}

export {
  install,
  update
}
