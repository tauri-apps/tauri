import { CargoManifest } from './../types/cargo'
import { existsSync, removeSync, writeFileSync } from 'fs-extra'
import { join, normalize, resolve } from 'path'
import { TauriConfig, TauriBuildConfig } from 'types'
import { merge } from 'webpack-merge'
import copyTemplates from '../helpers/copy-templates'
import logger from '../helpers/logger'
import defaultConfig from './defaultConfig'
import chalk from 'chalk'
import { allRecipes, Recipe } from '../api/recipes'
import { get } from 'lodash'

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
  customConfig: Partial<TauriConfig> = {}
): boolean | undefined => {
  const path = join(injectPath, 'tauri.conf.json')
  if (existsSync(path) && force !== 'conf' && force !== 'all') {
    warn(`tauri.conf.json found in ${path}
  Run \`tauri init --force conf\` to overwrite.`)
    if (!force) return false
  } else {
    removeSync(path)
    Object.keys(defaultConfig).forEach(key => {
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

    // If a recipe is present in the build config, then
    // we will run the custom config through the recipe's
    // modification to make sure any needed recipe config is applied.
    if (customConfig.build !== undefined) {
      const build: TauriBuildConfig = customConfig.build

      if (build.recipe !== undefined) {
        const recipeName: string = build.recipe

        const recipe: Recipe | undefined = get(allRecipes, recipeName, undefined)

        if (recipe !== undefined) {
          customConfig.build = recipe.configUpdate(build)
        }
      }
    }

    const finalConf = merge(defaultConfig as any, customConfig as any) as UnknownObject

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

  const resolveTauriPath = (tauriPath: string): string => {
    const resolvedPath = tauriPath.startsWith('/') || /^\S:/g.test(tauriPath)
      ? join(tauriPath, 'tauri') // we received a full path as argument
      : join('..', tauriPath, 'tauri') // we received a relative path
    return resolvedPath.replace(/\\/g, '/')
  }

  const resolveCurrentTauriVersion = (): string => {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-var-requires, @typescript-eslint/no-unsafe-member-access
    const tauriManifest = require('../../../../tauri/Cargo.toml') as CargoManifest
    const version = tauriManifest.package.version
    return version.substring(0, version.lastIndexOf('.'))
  }

  const tauriDep = tauriPath
    ? `{ path = "${resolveTauriPath(tauriPath)}" }`
    : `{ version = "${resolveCurrentTauriVersion()}" }`

  removeSync(dir)
  copyTemplates({
    source: resolve(__dirname, '../../templates/src-tauri'),
    scope: {
      tauriDep
    },
    target: dir
  })
  if (logging) log('Successfully wrote src-tauri')
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
