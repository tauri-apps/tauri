const { copySync, renameSync, existsSync, mkdirSync } = require('fs-extra'),
  { resolve, join, normalize } = require('path'),
  logger = require('./helpers/logger'),
  warn = logger('app:tauri (path)', 'red')

const inject = (injectPath, type) => {
  if (existsSync(injectPath)) {
    warn(`Tauri dir (${injectPath}) not empty.`)
    return false
  }

  switch (type) {
    case 'conf':
      const dir = normalize(join(injectPath, '..'))
      const path = join(dir, 'tauri.conf.js')
      if (existsSync(path)) {
        warn(`tauri.conf.js found in ${path}`)
        return false
      } else {
        copySync(resolve(__dirname, '../templates/conf/tauri.conf.js'), path)
      }
      break
    case 'template':
      mkdirSync(injectPath)
      copySync(resolve(__dirname, '../templates/rust'), injectPath)
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
        renameSync(join(injectPath, rawPath), join(injectPath, targetRelativePath))
      }
      break
    default:
      break
  }

  return true
}

module.exports = {
  inject
}
