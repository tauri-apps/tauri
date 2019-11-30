const { copySync, existsSync, removeSync, readFileSync } = require('fs-extra')
const { resolve, join, normalize } = require('path')
const copyTemplates = require('./helpers/copy-templates')

const logger = require('./helpers/logger')
const log = logger('app:tauri', 'green')
const warn = logger('app:tauri (template)', 'red')

const injectConfFile = (injectPath, { force, logging }) => {
  const path = join(injectPath, 'tauri.conf.js')
  if (existsSync(path) && force !== 'conf' && force !== 'all') {
    warn(`tauri.conf.js found in ${path}
  Run \`tauri init --force conf\` to overwrite.`)
    if (!force) return false
  } else {
    try {
      removeSync(path)
      copySync(resolve(__dirname, './templates/tauri.conf.js'), path)
    } catch (e) {
      if (logging) console.log(e)
      return false
    } finally {
      if (logging) log('Successfully wrote tauri.conf.js')
    }
  }
}

const injectTemplate = (injectPath, { force, logging, tauriPath }) => {
  const dir = normalize(join(injectPath, 'src-tauri'))
  if (existsSync(dir) && force !== 'template' && force !== 'all') {
    warn(`Tauri dir (${dir}) not empty.
Run \`tauri init --force template\` to overwrite.`)
    if (!force) return false
  }

  let tauriDep
  if (tauriPath) {
    tauriDep = `{ path = "${resolve(process.cwd(), tauriPath, 'tauri')}" }`
  } else {
    const toml = require('@tauri-apps/toml')
    const tomlPath = join(__dirname, '../../tauri/Cargo.toml')
    const tomlFile = readFileSync(tomlPath)
    const tomlContents = toml.parse(tomlFile)
    tauriDep = `{ version = "${tomlContents.package.version}" }`
  }

  try {
    removeSync(dir)
    copyTemplates({
      source: resolve(__dirname, './templates/src-tauri'),
      scope: {
        tauriDep
      },
      target: dir
    })
  } catch (e) {
    if (logging) console.log(e)
    return false
  }
}

/**
 *
 * @param {string} injectPath
 * @param {string} type ['conf'|'template'|'all']
 * @param {string|boolean} [force=false] - One of[false|'conf'|'template'|'all']
 * @param {boolean} [logging=false]
 * @param {string} [tauriPath=null]
 * @returns {boolean}
 */
const inject = (injectPath, type, { force = false, logging = false, tauriPath = null }) => {
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

module.exports = {
  inject
}
