import { existsSync, removeSync, writeFileSync } from 'fs-extra'
import { join, normalize, resolve } from 'path'
import { TauriConfig } from 'types'
import merge from 'webpack-merge'
import copyTemplates from '../helpers/copy-templates'
import logger from '../helpers/logger'
import defaultConfig from './defaultConfig'
import chalk from 'chalk'

const log = logger('app:tauri')
const warn = logger('app:tauri (template)', chalk.red)

interface InjectOptions {
  force: false | InjectionType
  logging: boolean
  tauriPath?: string
}
type InjectionType = 'conf' | 'template' | 'all'

const injectConfFile = (
  injectPath: string,
  { force, logging }: InjectOptions,
  customConfig: Partial<TauriConfig> = {}
): boolean | undefined => {
  const path = join(injectPath, 'tauri.conf.json')
  if (existsSync(path) && force !== 'conf' && force !== 'all') {
    warn(`tauri.conf.json found in ${path}
  Run \`tauri init --force conf\` to overwrite.`)
    if (!force) return false
  } else {
    try {
      removeSync(path)
      const finalConf = merge(defaultConfig as any, customConfig as any) as {
        [index: string]: any
      }
      Object.keys(finalConf).forEach(key => {
        // Options marked `null` should be removed
        /* eslint-disable security/detect-object-injection */
        if (finalConf[key] === null) {
          // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
          delete finalConf[key]
        }
        /* eslint-enable security/detect-object-injection */
      })
      writeFileSync(path, JSON.stringify(finalConf, undefined, 2))
    } catch (e) {
      if (logging) console.log(e)
      return false
    } finally {
      if (logging) log('Successfully wrote tauri.conf.json')
    }
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

  const resolveTauriPath = (tauriPath: string): string => {
    const resolvedPath = tauriPath.startsWith('/') || /^\S:/g.test(tauriPath)
      ? join(tauriPath, 'tauri') // we received a full path as argument
      : join('..', tauriPath, 'tauri') // we received a relative path
    return resolvedPath.replace(/\\/g, '/')
  }
  const tauriDep = tauriPath
    ? `{ path = "${resolveTauriPath(tauriPath)}" }`
    : null

  try {
    removeSync(dir)
    copyTemplates({
      source: resolve(__dirname, '../../templates/src-tauri'),
      scope: {
        tauriDep
      },
      target: dir
    })
  } catch (e) {
    if (logging) console.log(e)
    return false
  } finally {
    if (logging) log('Successfully wrote src-tauri')
  }
}

const inject = (
  injectPath: string,
  type: InjectionType,
  { force = false, logging = false, tauriPath }: InjectOptions,
  customConfig?: Partial<TauriConfig>
): boolean => {
  if (typeof type !== 'string' || typeof injectPath !== 'string') {
    warn('- internal error. Required params missing.')
    return false
  }
  if (type === 'template' || type === 'all') {
    injectTemplate(injectPath, { force, logging, tauriPath })
  }
  if (type === 'conf' || type === 'all') {
    injectConfFile(join(injectPath, 'src-tauri'), { force, logging }, customConfig)
  }
  return true
}

export { inject }
