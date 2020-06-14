import toml from '@tauri-apps/toml'
import chalk from 'chalk'
import { sync as spawn } from 'cross-spawn'
import fs from 'fs'
import os from 'os'
import path from 'path'
import { appDir, tauriDir } from '../helpers/app-paths'
import { TauriConfig } from './../types/config'
import { CargoLock, CargoManifest } from '../types/cargo'
import nonWebpackRequire from '../helpers/non-webpack-require'
import { version } from '../../package.json'

interface DirInfo {
  path: string
  name: string
  type?: 'folder' | 'file'
  children?: DirInfo[]
}

/* eslint-disable security/detect-non-literal-fs-filename */
function dirTree(filename: string, recurse = true): DirInfo {
  const stats = fs.lstatSync(filename)
  const info: DirInfo = {
    path: filename,
    name: path.basename(filename)
  }

  if (stats.isDirectory()) {
    info.type = 'folder'
    if (recurse) {
      info.children = fs.readdirSync(filename).map(function(child: string) {
        return dirTree(filename + '/' + child, false)
      })
    }
  } else {
    info.type = 'file'
  }

  return info
}

function getVersion(
  command: string,
  args: string[] = [],
  formatter?: (output: string) => string
): string {
  try {
    const child = spawn(command, [...args, '--version'])
    if (child.status === 0) {
      const output = String(child.output[1])
      return chalk
        .green(formatter === undefined ? output : formatter(output))
        .replace('\n', '')
    }
    return chalk.red('Not installed')
  } catch (err) {
    return chalk.red('Not installed')
  }
}

interface Info {
  section?: boolean
  key: string
  value?: string
}

function printInfo(info: Info): void {
  console.log(
    `${info.section ? '\n' : ''}${info.key}${
      info.value === undefined ? '' : ' - ' + info.value
    }`
  )
}

function readTomlFile<T extends CargoLock | CargoManifest>(filepath: string): T {
  if (fs.existsSync(filepath)) {
    const file = fs.readFileSync(filepath).toString()
    return toml.parse(file) as unknown as T
  } else {
    return false as unknown as T
  }
}

function printAppInfo(tauriDir: string): void {
  printInfo({ key: 'App', section: true })

  const lockPath = path.join(tauriDir, 'Cargo.lock')
  const lockContents = readTomlFile<CargoLock>(lockPath)
  let tauriPackages
  let tauriVersion

  if (lockContents) {
    tauriPackages = lockContents.package.filter(pkg => pkg.name === 'tauri')
    if (tauriPackages.length <= 0) {
      tauriVersion = chalk.red('unknown')
    } else if (tauriPackages.length === 1) {
      tauriVersion = chalk.green(tauriPackages[0].version)
    } else {
      // there are multiple `tauri` packages in the lockfile
      // load and check the manifest version to display alongside the found versions
      const manifestPath = path.join(tauriDir, 'Cargo.toml')
      const manifestContent = readTomlFile<CargoManifest>(manifestPath)

      const manifestVersion = (): string => {
        const tauri = manifestContent.dependencies.tauri
        if (tauri) {
          if (typeof tauri === 'string') {
            return chalk.yellow(tauri)
          } else if (tauri.version) {
            return chalk.yellow(tauri.version)
          } else if (tauri.path) {
            const manifestPath = path.resolve(tauriDir, tauri.path, 'Cargo.toml')
            const manifestContent = readTomlFile<CargoManifest>(manifestPath)
            const pathVersion = chalk.yellow(manifestContent.package.version)
            return `path:${tauri.path} [${pathVersion}]`
          }
        }
        return chalk.red('unknown')
      }

      const versions = tauriPackages.map(p => p.version).join(', ')
      tauriVersion = `${manifestVersion()} (${chalk.yellow(versions)})`
    }
  } else {
    const tomlPath = path.join(tauriDir, 'Cargo.toml')
    const tomlFile = fs.readFileSync(tomlPath).toString()
    const tomlContents = toml.parse(tomlFile) as any as CargoManifest
    const tauri = tomlContents.dependencies.tauri
    if (tauri) {
      if (typeof tauri === 'string') {
        tauriVersion = chalk.green(tauri)
      } else if (tauri.version) {
        tauriVersion = chalk.green(tauri.version)
      } else if (tauri.path) {
        try {
          const tauriTomlPath = path.resolve(
            tauriDir,
            tauri.path,
            'Cargo.toml'
          )
          const tauriTomlFile = fs.readFileSync(tauriTomlPath).toString()
          const tauriTomlContents = toml.parse(tauriTomlFile) as any as CargoManifest
          tauriVersion = chalk.green(
            // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
            `${tauriTomlContents.package.version} (from source)`
          )
        } catch (_) {
          tauriVersion = chalk.red('unknown')
        }
      } else {
        tauriVersion = chalk.red('unknown')
      }
    }
  }
  printInfo({ key: '  tauri.rs', value: tauriVersion })

  try {
    const tauriMode = (config: TauriConfig): string => {
      if (config.tauri.embeddedServer) {
        return chalk.green(
          config.tauri.embeddedServer.active ? 'embedded-server' : 'no-server'
        )
      }
      return chalk.red('unset')
    }
    const configPath = path.join(tauriDir, 'tauri.conf.json')
    const config = nonWebpackRequire(configPath) as TauriConfig
    printInfo({ key: '  mode', value: tauriMode(config) })
    printInfo({
      key: '  build-type',
      value:
        config.tauri.bundle?.active ? 'bundle' : 'build'
    })
    printInfo({
      key: '  CSP',
      value: config.tauri.security ? config.tauri.security.csp : 'unset'
    })
    printInfo({
      key: '  Windows',
      value: config.tauri.edge?.active ? 'Edge' : 'MSHTML'
    })
    printInfo({
      key: '  distDir',
      value: config.build
        ? chalk.green(config.build.distDir)
        : chalk.red('unset')
    })
    printInfo({
      key: '  devPath',
      value: config.build
        ? chalk.green(config.build.devPath)
        : chalk.red('unset')
    })
  } catch (_) {}
}

module.exports = () => {
  printInfo({
    key: 'Operating System',
    value: chalk.green(
      `${os.type()}(${os.release()}) - ${os.platform()}/${os.arch()}`
    ),
    section: true
  })
  printInfo({ key: 'Node.js environment', section: true })
  printInfo({ key: '  Node.js', value: chalk.green(process.version.slice(1)) })
  printInfo({
    key: '  tauri.js',
    value: chalk.green(version)
  })
  printInfo({ key: 'Rust environment', section: true })
  printInfo({
    key: '  rustc',
    value: getVersion('rustc', [], output => output.split(' ')[1])
  })
  printInfo({
    key: '  cargo',
    value: getVersion('cargo', [], output => output.split(' ')[1])
  })
  printInfo({ key: '  tauri-bundler', value: getVersion('cargo', ['tauri-bundler']) })
  printInfo({ key: 'Global packages', section: true })
  printInfo({ key: '  NPM', value: getVersion('npm') })
  printInfo({ key: '  yarn', value: getVersion('yarn') })
  printInfo({ key: 'App directory structure', section: true })

  const tree = dirTree(appDir)
  // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
  for (const artifact of tree.children || []) {
    if (artifact.type === 'folder') {
      console.log(`/${artifact.name}`)
    }
  }
  printAppInfo(tauriDir)
}

/* eslint-enable security/detect-non-literal-fs-filename */
