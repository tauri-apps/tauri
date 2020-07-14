import { spawnSync } from './../../helpers/spawn'
import { CargoManifest, CargoManifestDependency } from './../../types/cargo'
import { ManagementType, Result } from './types'
import { getCrateLatestVersion, semverLt } from './util'
import logger from '../../helpers/logger'
import { resolve as appResolve, tauriDir } from '../../helpers/app-paths'
import { readFileSync, writeFileSync, existsSync } from 'fs'
import toml from '@tauri-apps/toml'
import inquirer from 'inquirer'

const log = logger('dependency:crates')

const dependencies = ['tauri']

function getManifest(): CargoManifest {
  const manifest = readFileSync(appResolve.tauri('Cargo.toml')).toString()
  return toml.parse(manifest) as any as CargoManifest
}

function dependencyDefinition(version: string): CargoManifestDependency {
  return { version: version.substring(0, version.lastIndexOf('.')) }
}

async function manageDependencies(managementType: ManagementType): Promise<Result> {
  const installedDeps = []
  const updatedDeps = []

  const manifest = getManifest()

  for (const dependency of dependencies) {
    // eslint-disable-next-line security/detect-object-injection
    const manifestDep = manifest.dependencies[dependency]
    // TODO current version should be read from Cargo.lock if it exists
    const currentVersion = typeof manifestDep === 'string' ? manifestDep : manifestDep?.version
    if (currentVersion === undefined) {
      log(`Installing ${dependency}...`)
      const latestVersion = await getCrateLatestVersion(dependency)
      // eslint-disable-next-line security/detect-object-injection
      manifest.dependencies[dependency] = dependencyDefinition(latestVersion)
      installedDeps.push(dependency)
    } else if (managementType === ManagementType.Update) {
      const latestVersion = await getCrateLatestVersion(dependency)
      if (semverLt(currentVersion, latestVersion)) {
        const inquired = await inquirer.prompt([{
          type: 'confirm',
          name: 'answer',
          message: `[CRATES] ${dependency} latest version is ${latestVersion}. Do you want to update?`,
          default: false
        }])
        if (inquired.answer) {
          log(`Updating ${dependency}...`)
          // eslint-disable-next-line security/detect-object-injection
          manifest.dependencies[dependency] = dependencyDefinition(latestVersion)
          updatedDeps.push(dependency)
        }
      } else {
        log(`${dependency} is up to date`)
      }
    } else {
      log(`${dependency} is already installed`)
    }
  }

  if (installedDeps.length || updatedDeps.length) {
    writeFileSync(appResolve.tauri('Cargo.toml'), toml.stringify(manifest as any))
  }
  if (updatedDeps.length) {
    if (!existsSync(appResolve.tauri('Cargo.lock'))) {
      spawnSync('cargo', ['generate-lockfile'], tauriDir)
    }
    spawnSync('cargo', ['update', ...updatedDeps.reduce<string[]>((initialValue, dep) => [...initialValue, '-p', dep], [])], tauriDir)
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

export {
  install,
  update
}
