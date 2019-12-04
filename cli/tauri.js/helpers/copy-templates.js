// forked from https://github.com/quasarframework/quasar/blob/master/app/lib/app-extension/Extension.js
function renderFolders ({ source, target, scope }) {
  const
    fs = require('fs-extra')
  const { join, resolve } = require('path')
  const fglob = require('fast-glob')
  const isBinary = require('isbinaryfile').isBinaryFileSync
  const compileTemplate = require('lodash.template')

  const files = fglob.sync(['**/*'], {
    cwd: source
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

    const targetPath = join(target, targetRelativePath)
    const sourcePath = resolve(source, rawPath)

    fs.ensureFileSync(targetPath)

    if (isBinary(sourcePath)) {
      fs.copyFileSync(sourcePath, targetPath)
    } else {
      const rawContent = fs.readFileSync(sourcePath, 'utf-8')
      const template = compileTemplate(rawContent, {
        interpolate: /<%=([\s\S]+?)%>/g
      })
      fs.writeFileSync(targetPath, template(scope), 'utf-8')
    }
  }
}

module.exports = renderFolders
