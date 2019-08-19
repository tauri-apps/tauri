const { copySync, renameSync, existsSync, mkdirSync } = require('fs-extra'),
  path = require('path')

class TauriInjector {
  constructor(appPaths) {
    this.appPaths = appPaths
  }

  configDir() {
    return path.resolve(__dirname, '..')
  }

  injectTemplate() {
    if (existsSync(this.appPaths.tauriDir)) {
      console.log(`Tauri dir (${this.appPaths.tauriDir}) not empty.`)
      return false
    }
    mkdirSync(this.appPaths.tauriDir)
    copySync(path.resolve(__dirname, '../templates/rust'), this.appPaths.tauriDir)
    const files = require('fast-glob').sync(['**/_*'], {
      cwd: this.appPaths.tauriDir
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
      renameSync(this.appPaths.resolve.tauri(rawPath), this.appPaths.resolve.tauri(targetRelativePath))
    }
    return true
  }
}

module.exports = TauriInjector
