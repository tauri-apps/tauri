import { existsSync } from 'fs'
import { resolve, join } from 'path'
import { spawnSync, spawn } from './spawn'
import { CargoManifest } from '../types/cargo'
import { readTomlFile } from '../helpers/toml'

const currentTauriCliVersion = (): string => {
  const manifestPath = join(__dirname, '../../../core/Cargo.toml')
  const tauriCliManifest = readTomlFile<CargoManifest>(manifestPath)
  const version = tauriCliManifest?.package.version
  if (version !== undefined) {
    return version
  }
  throw Error('Unable to parse latest CLI version')
}

export function runOnRustCli(
  command: string,
  args: string[]
): { pid: number; promise: Promise<void> } {
  const targetPath = resolve(__dirname, '../..')
  const targetCliPath = join(targetPath, 'bin/cargo-tauri')

  let resolveCb: () => void
  let rejectCb: () => void
  let pid: number
  const promise = new Promise<void>((resolve, reject) => {
    resolveCb = resolve
    rejectCb = () => reject(new Error())
  })
  const onClose = (code: number, pid: number): void => {
    if (code === 0) {
      resolveCb()
    } else {
      rejectCb()
    }
  }

  if (existsSync(targetCliPath)) {
    pid = spawn(
      targetCliPath,
      ['tauri', command, ...args],
      process.cwd(),
      onClose
    )
  } else {
    if (existsSync(resolve(targetPath, '../tauri-bundler'))) {
      // running local CLI
      const cliPath = resolve(targetPath, '../core')
      spawnSync('cargo', ['build', '--release'], cliPath)
      const localCliPath = resolve(
        targetPath,
        '../core/target/release/cargo-tauri'
      )
      pid = spawn(
        localCliPath,
        ['tauri', command, ...args],
        process.cwd(),
        onClose
      )
    } else {
      spawnSync(
        'cargo',
        [
          'install',
          '--root',
          targetPath,
          'tauri-cli',
          '--version',
          currentTauriCliVersion()
        ],
        process.cwd()
      )
      pid = spawn(
        targetCliPath,
        ['tauri', command, ...args],
        process.cwd(),
        onClose
      )
    }
  }

  return { pid, promise }
}
