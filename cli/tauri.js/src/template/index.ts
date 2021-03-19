import { CargoManifest } from './../types/cargo'
import { existsSync, removeSync, writeFileSync } from 'fs-extra'
import { join, normalize, resolve, isAbsolute } from 'path'
import { merge } from 'webpack-merge'
import copyTemplates from '../helpers/copy-templates'
import logger from '../helpers/logger'
import defaultConfig from './defaultConfig'
import { readTomlFile } from '../helpers/toml'
import chalk from 'chalk'

const log = logger('app:tauri')
const warn = logger('app:tauri (template)', chalk.red)

interface InjectOptions {
  force: false | InjectionType
  logging: boolean
  tauriPath?: string
}
type InjectionType = 'conf' | 'template' | 'all'

interface UnknownObject {
  [index: string]: any
}

const injectConfFile = (
  injectPath: string,
  { force, logging }: InjectOptions,
  customConfig: Object = {}
): boolean | undefined => {
  const path = join(injectPath, 'tauri.conf.json')
  if (existsSync(path) && force !== 'conf' && force !== 'all') {
    warn(`tauri.conf.json found in ${path}
  Run \`tauri init --force conf\` to overwrite.`)
    if (!force) return false
  } else {
    removeSync(path)
    Object.keys(defaultConfig).forEach((key) => {
      // Options marked `null` should be removed
      /* eslint-disable security/detect-object-injection */
      if ((customConfig as UnknownObject)[key] === null) {
        // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
        delete (defaultConfig as UnknownObject)[key]
        // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
        delete (customConfig as UnknownObject)[key]
      }
      /* eslint-enable security/detect-object-injection */
    })
    // Window config should be merged
    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
    if ((customConfig as UnknownObject).tauri?.windows[0]) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-assignment
      ;(customConfig as UnknownObject).tauri.windows[0] = {
        ...defaultConfig.tauri.windows[0],
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
        ...(customConfig as UnknownObject).tauri.windows[0]
      }
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      delete (defaultConfig as UnknownObject).tauri.windows
    }
    const finalConf = merge(
      defaultConfig as any,
      customConfig as any
    ) as UnknownObject

    writeFileSync(path, JSON.stringify(finalConf, undefined, 2))
    if (logging) log('Successfully wrote tauri.conf.json')
  }
}

const injectTemplate = (
  injectPath: string,
  { force, logging, tauriPath }: InjectOptions
): boolean | undefined => {
  const dir = normalize(join(injectPath, 'src-tauri'))
  if (existsSync(dir) && force !== 'template' && force !== 'all') {
    warn(`Tauri dir (${dir}) not empty.
Run \`tauri init --force template\` to overwrite.`)
    if (!force) return false
  }

  const resolveTauriPath = (tauriPath: string, crate: string): string => {
    const resolvedPath = isAbsolute(tauriPath)
      ? join(tauriPath, crate) // we received a full path as argument
      : join('..', tauriPath, crate) // we received a relative path
    return resolvedPath.replace(/\\/g, '/')
  }

  const resolveCurrentTauriVersion = (crate: string): string => {
    const manifestPath = join(__dirname, `../../../../${crate}/Cargo.toml`)
    const tauriManifest = readTomlFile<CargoManifest>(manifestPath)
    const version = tauriManifest?.package.version
    if (version !== undefined) {
      return version.substring(0, version.lastIndexOf('.'))
    }
    throw Error('Unable to parse latest tauri version')
  }

  const tauriDep = tauriPath
    ? `{ path = "${resolveTauriPath(tauriPath, 'tauri')}" }`
    : `{ version = "${resolveCurrentTauriVersion('tauri')}" }`
  const tauriBuildDep = tauriPath
    ? `{ path = "${resolveTauriPath(tauriPath, 'core/tauri-build')}" }`
    : `{ version = "${resolveCurrentTauriVersion('core/tauri-build')}" }`

  removeSync(dir)
  copyTemplates({
    source: resolve(__dirname, '../../templates/src-tauri'),
    scope: {
      tauriDep,
      tauriBuildDep
    },
    target: dir
  })
  if (logging) log('Successfully wrote src-tauri')
}

const inject = (
  injectPath: string,
  type: InjectionType,
  { force = false, logging = false, tauriPath }: InjectOptions,
  customConfig?: Object
): boolean => {
  if (typeof type !== 'string' || typeof injectPath !== 'string') {
    warn('- internal error. Required params missing.')
    return false
  }
  if (type === 'template' || type === 'all') {
    injectTemplate(injectPath, { force, logging, tauriPath })
  }
  if (type === 'conf' || type === 'all') {
    injectConfFile(
      join(injectPath, 'src-tauri'),
      { force, logging },
      customConfig
    )
  }
  return true
}

export { inject }
