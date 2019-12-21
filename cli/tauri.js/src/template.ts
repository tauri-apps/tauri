import { copySync, existsSync, removeSync } from 'fs-extra'
import { join, normalize, resolve } from 'path'
import copyTemplates from './helpers/copy-templates'
import logger from './helpers/logger'

const log = logger('app:tauri', 'green')
const warn = logger('app:tauri (template)', 'red')

const injectConfFile = (
  injectPath: string,
  {
    force,
    logging
  }: {
    force: false | 'conf' | 'template' | 'all'
    logging: boolean
  }
): boolean | undefined => {
  const path = join(injectPath, 'tauri.conf.js')
  if (existsSync(path) && force !== 'conf' && force !== 'all') {
    warn(`tauri.conf.js found in ${path}
  Run \`tauri init --force conf\` to overwrite.`)
    if (!force) return false
  } else {
    try {
      removeSync(path)
      copySync(resolve(__dirname, '../templates/tauri.conf.js'), path)
    } catch (e) {
      if (logging) console.log(e)
      return false
    } finally {
      if (logging) log('Successfully wrote tauri.conf.js')
    }
  }
}

const injectTemplate = (
  injectPath: string,
  {
    force,
    logging,
    tauriPath
  }: {
    force: false | 'conf' | 'template' | 'all'
    logging: boolean
    tauriPath?: string
  }
): boolean | undefined => {
  const dir = normalize(join(injectPath, 'src-tauri'))
  if (existsSync(dir) && force !== 'template' && force !== 'all') {
    warn(`Tauri dir (${dir}) not empty.
Run \`tauri init --force template\` to overwrite.`)
    if (!force) return false
  }

  const tauriDep = tauriPath
    ? `{ path = "${join('..', tauriPath, 'tauri')}" }`
    : null

  try {
    removeSync(dir)
    copyTemplates({
      source: resolve(__dirname, '../templates/src-tauri'),
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
  type: 'conf' | 'template' | 'all',
  {
    force = false,
    logging = false,
    tauriPath
  }: {
    force: false | 'conf' | 'template' | 'all'
    logging: boolean
    tauriPath?: string
  }
): boolean => {
  if (typeof type !== 'string' || typeof injectPath !== 'string') {
    warn('- internal error. Required params missing.')
    return false
  }
  if (type === 'conf' || type === 'all') {
    injectConfFile(injectPath, { force, logging })
  }
  if (type === 'template' || type === 'all') {
    injectTemplate(injectPath, { force, logging, tauriPath })
  }
  return true
}

export { inject }
