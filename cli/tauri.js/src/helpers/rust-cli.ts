import { existsSync } from 'fs'
import { resolve, join } from 'path'
import { spawnSync } from './spawn'
import { CargoManifest } from '../types/cargo'

const currentTauriCliVersion = (): string => {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-var-requires, @typescript-eslint/no-unsafe-member-access
  const tauriCliManifest = require('../../../Cargo.toml') as CargoManifest
  return tauriCliManifest.package.version
}

export function runOnRustCli(command: string, args: string[]): void {
  const targetPath = resolve(__dirname, '../..')
  const targetCliPath = join(targetPath, 'bin/cargo-tauri')
  if (existsSync(targetCliPath)) {
    spawnSync(targetCliPath, ['tauri', command, ...args], process.cwd())
  } else {
    const localCliPath = resolve(targetPath, '../target/release/cargo-tauri')
    if (existsSync(resolve(targetPath, '../tauri-bundler'))) { // running local CLI
      const cliPath = resolve(targetPath, '..')
      spawnSync('cargo', ['build', '--release'], cliPath)
      spawnSync(localCliPath, ['tauri', command, ...args], process.cwd())
    } else {
      spawnSync('cargo', ['install', '--root', targetPath, 'tauri-cli', '--version', currentTauriCliVersion()], process.cwd())
      spawnSync(targetCliPath, ['tauri', command, ...args], process.cwd())
    }
  }
}
