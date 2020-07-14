import { ManagementType } from './types'
import { getCrateLatestVersion } from './util'
import getScriptVersion from '../../helpers/get-script-version'
import logger from '../../helpers/logger'
import { sync as spawnSync } from 'cross-spawn'
import semver from 'semver'
import inquirer from 'inquirer'

const log = logger('dependency:cargo-commands')

const dependencies = ['tauri-bundler']

async function manageDependencies(managementType: ManagementType) {
  for (const dependency of dependencies) {
    const currentVersion = getScriptVersion('cargo', [dependency])
    if (currentVersion === null) {
      log(`Installing ${dependency}...`)
      spawnSync('cargo', ['install', dependency])
    } else if (managementType === ManagementType.Update) {
      const latestVersion = await getCrateLatestVersion(dependency)
      if (semver.lt(currentVersion, latestVersion)) {
        const { answer } = await inquirer.prompt([{
          type: 'confirm',
          name: 'answer',
          message: `${dependency} latest version is ${latestVersion}. Do you want to update?`,
          default: false
        }])
        if (answer) {
          spawnSync('cargo', ['install', dependency, '--force'])
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
