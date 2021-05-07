import { IManager } from './types'
import { sync as crossSpawnSync } from 'cross-spawn'
import { spawnSync } from '../../../helpers/spawn'
import { appDir } from '../../../helpers/app-paths'

export class YarnManager implements IManager {
  type = 'yarn'

  installPackage(packageName: string): void {
    spawnSync('yarn', ['add', packageName], appDir)
  }

  installDevPackage(packageName: string): void {
    spawnSync('yarn', ['add', packageName, '--dev'], appDir)
  }

  updatePackage(packageName: string): void {
    spawnSync('yarn', ['upgrade', packageName, '--latest'], appDir)
  }

  getPackageVersion(packageName: string): string | null {
    const child = crossSpawnSync(
      'yarn',
      ['list', '--pattern', packageName, '--depth', '0'],
      { cwd: appDir }
    )

    const output = String(child.output[1])
    // eslint-disable-next-line security/detect-non-literal-regexp
    const matches = new RegExp(packageName + '@(\\S+)', 'g').exec(output)
    if (matches?.[1]) {
      return matches[1]
    } else {
      return null
    }
  }

  getLatestVersion(packageName: string): string {
    const child = crossSpawnSync(
      'yarn',
      ['info', packageName, 'versions', '--json'],
      { cwd: appDir }
    )
    const output = String(child.output[1])
    const packageJson = JSON.parse(output) as { data: string[] }
    return packageJson.data[packageJson.data.length - 1]
  }
}
