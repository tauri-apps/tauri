const { copySync, renameSync, existsSync, mkdirSync, removeSync } = require('fs-extra'),
  { resolve, join, normalize } = require('path'),
  logger = require('./helpers/logger'),
  log = logger('app:tauri', 'green'),
  warn = logger('app:tauri (template)', 'red')

const injectConfFile = (injectPath, force, logging, directory) => {
  const dir = normalize(join(injectPath, '..'))
  const path = join(dir, 'tauri.conf.js')
  if (existsSync(path) && force !== 'conf' && force !== 'all') {
    warn(`tauri.conf.js found in ${path}
  Run \`tauri init --force conf\` to overwrite.`)
    if (!force) return false
  } else {
    try {
      removeSync(path)
      copySync(resolve(__dirname, '../templates/conf/tauri.conf.js'), path)
    } catch (e) {
      if (logging) console.log(e)
      return false
    } finally {
      if (logging) log('Successfully wrote tauri.conf.js')
    }
  }
}

const injectTemplate = (injectPath, force, logging, directory) => {
  if (existsSync(injectPath) && force !== 'template' && force !== 'all') {
    warn(`Tauri dir (${injectPath}) not empty.
Run \`tauri init --force template\` to overwrite.`)
    if (!force) return false
  }
  try {
    removeSync(injectPath)
    mkdirSync(injectPath)
    copySync(resolve(__dirname, '../templates/rust'), injectPath)
  } catch (e) {
    if (logging) console.log(e)
    return false
  }
  const files = require('fast-glob').sync(['**/_*'], {
    cwd: injectPath
  })
  for (const rawPath of files) {
    const targetRelativePath = rawPath.split('/').map(name => {
      // dotfiles are ignored when published to npm, therefore in templates
      // we need to use underscore instead (e.g. "_gitignore")
      if (name.charAt(0) === '_' && name.charAt(1) !== '_') {
        return `.${name.slice(1)}`
      }
      if (name.charAt(0) === '_' && name.charAt(1) === '_') {
        return `${name.slice(1)}`
      }
      return name
    }).join('/')
    try {
      renameSync(join(injectPath, rawPath), join(injectPath, targetRelativePath))
    } catch (e) {
      if (logging) console.log(e)
      return false
    } finally {
      if (logging) log('Successfully wrote tauri template files')
    }
  }
}

/**
 *
 * @param {string} injectPath
 * @param {string} type ['conf'|'template'|'all']
 * @param {string|boolean} [force=false] - One of[false|'conf'|'template'|'all']
 * @param {boolean} [logging=false]
 * @param {string} directory
 * @returns {boolean}
 */
const inject = (injectPath, type, force = false, logging = false, directory) => {
  if (typeof type !== 'string' || typeof injectPath !== 'string') {
    warn('- internal error. Required params missing.')
    return false
  }
  if (type === 'conf' || type === 'all') {
    injectConfFile(injectPath, force, logging, directory)
  }
  if (type === 'template' || type === 'all') {
    injectTemplate(injectPath, force, logging, directory)
  }
  return true
}

module.exports = {
  inject
}
