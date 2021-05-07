import { IManager } from './types'
import { sync as crossSpawnSync } from 'cross-spawn'
import { spawnSync } from '../../../helpers/spawn'
import { appDir } from '../../../helpers/app-paths'

export class NpmManager implements IManager {
  type = 'npm'

  installPackage(packageName: string): void {
    spawnSync('npm', ['install', packageName], appDir)
  }

  installDevPackage(packageName: string): void {
    spawnSync('npm', ['install', packageName, '--save-dev'], appDir)
  }

  updatePackage(packageName: string): void {
    spawnSync('npm', ['install', `${packageName}@latest`], appDir)
  }

  getPackageVersion(packageName: string): string | null {
    const child = crossSpawnSync(
      'npm',
      ['list', packageName, 'version', '--depth', '0'],
      {
        cwd: appDir
      }
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
    const child = crossSpawnSync('npm', ['show', packageName, 'version'], {
      cwd: appDir
    })

    return String(child.output[1]).replace('\n', '')
  }
}
