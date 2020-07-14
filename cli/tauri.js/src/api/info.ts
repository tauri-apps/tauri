
import toml from '@tauri-apps/toml'
import chalk from 'chalk'
import fs from 'fs'
import os from 'os'
import path from 'path'
import { appDir, tauriDir } from '../helpers/app-paths'
import { sync as spawn } from 'cross-spawn'
import { TauriConfig } from './../types/config'
import { CargoLock, CargoManifest } from '../types/cargo'
import nonWebpackRequire from '../helpers/non-webpack-require'
import { version } from '../../package.json'
import getScriptVersion from '../helpers/get-script-version'

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
      info.children = fs.readdirSync(filename).map(function (child: string) {
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
  const version = getScriptVersion(command, args)
  if (version === null) {
    return chalk.red('Not installed')
  } else {
    return chalk
      .green(formatter === undefined ? version : formatter(version))
      .replace('\n', '')
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

function readTomlFile<T extends CargoLock | CargoManifest>(filepath: string): T | null {
  try {
    const file = fs.readFileSync(filepath).toString()
    return toml.parse(file) as unknown as T
  } catch (_) {
    return null
  }
}

function printAppInfo(tauriDir: string): void {
  printInfo({ key: 'App', section: true })

  const lockPath = path.join(tauriDir, 'Cargo.lock')
  const lock = readTomlFile<CargoLock>(lockPath)
  const lockPackages = lock ? lock.package.filter(pkg => pkg.name === 'tauri') : []

  const manifestPath = path.join(tauriDir, 'Cargo.toml')
  const manifest = readTomlFile<CargoManifest>(manifestPath)

  let tauriVersion
  if (manifest && lock && lockPackages.length === 1) {
    // everything looks good
    tauriVersion = chalk.green(lockPackages[0].version)
  } else if (lock && lockPackages.length === 1) {
    // good lockfile, but no manifest - will cause problems building
    tauriVersion = `${chalk.green(lockPackages[0].version)} (${chalk.red('no manifest')})`
  } else {
    // we found multiple/none `tauri` packages in the lockfile, or
    // no manifest. in both cases we want more info on the manifest
    const manifestVersion = (): string => {
      const tauri = manifest?.dependencies.tauri
      if (tauri) {
        if (typeof tauri === 'string') {
          return chalk.yellow(tauri)
        } else if (tauri.version) {
          return chalk.yellow(tauri.version)
        } else if (tauri.path) {
          const manifestPath = path.resolve(tauriDir, tauri.path, 'Cargo.toml')
          const manifestContent = readTomlFile<CargoManifest>(manifestPath)
          let pathVersion = manifestContent?.package.version
          pathVersion = pathVersion ? chalk.yellow(pathVersion) : chalk.red(pathVersion)
          return `path:${tauri.path} [${pathVersion}]`
        }
      } else {
        return chalk.red('no manifest')
      }
      return chalk.red('unknown manifest')
    }

    let lockVersion
    if (lock && lockPackages.length > 0) {
      lockVersion = chalk.yellow(lockPackages.map(p => p.version).join(', '))
    } else if (lock && lockPackages.length === 0) {
      lockVersion = chalk.red('unknown lockfile')
    } else {
      lockVersion = chalk.red('no lockfile')
    }

    tauriVersion = `${manifestVersion()} (${chalk.yellow(lockVersion)})`
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
  } catch (_) { }
}

module.exports = () => {
  printInfo({
    key: 'Operating System',
    value: chalk.green(
      `${os.type()}(${os.release()}) - ${os.platform()}/${os.arch()}`
    ),
    section: true
  })
  if (os.platform() === 'win32') {
    const { stdout } = spawn('REG', ['QUERY', 'HKEY_CLASSES_root\\AppX3xxs313wwkfjhythsb8q46xdsq8d2cvv\\Application', '/v', 'ApplicationName'])
    const match = /{(\S+)}/g.exec(stdout.toString())
    if (match) {
      const edgeString = match[1]
      printInfo({ key: 'Microsoft Edge', value: edgeString.split('?')[0].replace('Microsoft.MicrosoftEdge_', '') })
    }
  }

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
