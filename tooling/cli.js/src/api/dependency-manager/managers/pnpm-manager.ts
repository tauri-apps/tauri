import { IManager } from './types'
import { sync as crossSpawnSync } from 'cross-spawn'
import { spawnSync } from '../../../helpers/spawn'
import { appDir } from '../../../helpers/app-paths'

export class PnpmManager implements IManager {
  type = 'pnpm'

  installPackage(packageName: string): void {
    spawnSync('pnpm', ['add', packageName], appDir)
  }

  installDevPackage(packageName: string): void {
    spawnSync('pnpm', ['add', packageName, '--save-dev'], appDir)
  }

  updatePackage(packageName: string): void {
    spawnSync('pnpm', ['add', `${packageName}@latest`], appDir)
  }

  getPackageVersion(packageName: string): string | null {
    const child = crossSpawnSync(
      'pnpm',
      ['list', packageName, 'version', '--depth', '0'],
      {
        cwd: appDir
      }
    )

    const output = String(child.output[1])
    // eslint-disable-next-line security/detect-non-literal-regexp
    const matches = new RegExp(packageName + ' (\\S+)', 'g').exec(output)

    if (matches?.[1]) {
      return matches[1]
    } else {
      return null
    }
  }

  getLatestVersion(packageName: string): string {
    const child = crossSpawnSync('pnpm', ['info', packageName, 'version'], {
      cwd: appDir
    })

    return String(child.output[1]).replace('\n', '')
  }
}
