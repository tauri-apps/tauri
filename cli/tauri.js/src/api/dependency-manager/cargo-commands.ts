import { ManagementType, Result } from './types'
import { getCrateLatestVersion, semverLt } from './util'
import getScriptVersion from '../../helpers/get-script-version'
import logger from '../../helpers/logger'
import { sync as spawnSync } from 'cross-spawn'
import inquirer from 'inquirer'

const log = logger('dependency:cargo-commands')

const dependencies = ['tauri-bundler']

async function manageDependencies(
  managementType: ManagementType
): Promise<Result> {
  const installedDeps = []
  const updatedDeps = []

  for (const dependency of dependencies) {
    const currentVersion = getScriptVersion('cargo', [dependency])
    if (currentVersion === null) {
      log(`Installing ${dependency}...`)
      spawnSync('cargo', ['install', dependency])
      installedDeps.push(dependency)
    } else if (managementType === ManagementType.Update) {
      const latestVersion = await getCrateLatestVersion(dependency)
      if (semverLt(currentVersion, latestVersion)) {
        const inquired = (await inquirer.prompt([
          {
            type: 'confirm',
            name: 'answer',
            message: `[CARGO COMMANDS] "${dependency}" latest version is ${latestVersion}. Do you want to update?`,
            default: false
          }
        ])) as { answer: boolean }
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
        if (inquired.answer) {
          spawnSync('cargo', ['install', dependency, '--force'])
          updatedDeps.push(dependency)
        }
      } else {
        log(`"${dependency}" is up to date`)
      }
    } else {
      log(`"${dependency}" is already installed`)
    }
  }

  const result: Result = new Map<ManagementType, string[]>()
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
