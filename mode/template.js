const { copySync, renameSync, existsSync, mkdirSync } = require('fs-extra'),
  path = require('path')

module.exports.inject = injectPath => {
  if (existsSync(injectPath)) {
    console.log(`Tauri dir (${injectPath}) not empty.`)
    return false
  }
  mkdirSync(injectPath)
  copySync(path.resolve(__dirname, '../templates/rust'), injectPath)
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
    renameSync(path.join(injectPath, rawPath), path.join(injectPath, targetRelativePath))
  }
  return true
}
