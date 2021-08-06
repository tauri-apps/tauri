// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { ManagementType, Result } from './types/deps'
import { shell } from './shell'

export type PackageManager = 'npm' | 'yarn' | 'pnpm'

export async function install({
  appDir,
  dependencies,
  devDependencies,
  packageManager
}: {
  appDir: string
  dependencies: string[]
  devDependencies: string[]
  packageManager: PackageManager
}): Promise<Result> {
  const result: Result = new Map<ManagementType, string[]>()
  await installNpmDevPackage(devDependencies, packageManager, appDir)
  result.set(ManagementType.Install, devDependencies)

  await installNpmPackage(dependencies, packageManager, appDir)
  result.set(ManagementType.Install, dependencies)

  return result
}

export async function checkPackageManager({
  cwd,
  packageManager
}: {
  cwd: string
  packageManager: PackageManager
}): Promise<boolean> {
  try {
    await shell(packageManager, ['--version'], { stdio: 'pipe', cwd })
    return true
  } catch (error) {
    throw new Error(
      `Must have ${packageManager} installed to manage dependencies. Is either in your PATH? We tried running in ${cwd}`
    )
  }
}

async function installNpmPackage(
  packageNames: string[],
  packageManager: PackageManager,
  appDir: string
): Promise<void> {
  const packages = packageNames.filter((p) => p !== '')
  if (packages.length !== 0) {
    console.log(`- Installing ${packages.join(', ')}...`)
    if (packageManager === 'npm') {
      await shell('npm', ['install', packageNames.join(' ')], {
        cwd: appDir
      })
    } else {
      await shell(packageManager, ['add', packageNames.join(' ')], {
        cwd: appDir
      })
    }
  }
}

async function installNpmDevPackage(
  packageNames: string[],
  packageManager: PackageManager,
  appDir: string
): Promise<void> {
  const packages = packageNames.filter((p) => p !== '')
  if (packages.length !== 0) {
    console.log(`- Installing ${packages.join(', ')}...`)
    if (packageManager === 'npm') {
      await shell(
        'npm',
        ['install', '--save-dev', '--ignore-scripts', packageNames.join(' ')],
        {
          cwd: appDir
        }
      )
    } else {
      await shell(
        packageManager,
        ['add', '-D', '--ignore-scripts', packageNames.join(' ')],
        {
          cwd: appDir
        }
      )
    }
  }
}
