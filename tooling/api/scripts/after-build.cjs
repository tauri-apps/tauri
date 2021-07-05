// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const { readFileSync, readdirSync, writeFileSync, copyFileSync } = require('fs')
const { copySync } = require('fs-extra')

/**
 * append our api modules to `exports` in `package.json` then write it to `./dist`
 */
const pkg = JSON.parse(readFileSync('package.json', 'utf8'))
const modules = readdirSync('src').map((mod) => mod.replace('.ts', ''))
if (!pkg.exports) {
  pkg.exports = {}
}

const outputPkg = {
  ...pkg,
  exports: Object.assign(
    {},
    ...modules.map((mod) => {
      let temp = {}
      let key = `./${mod}`
      if (mod === 'index') {
        key = '.'
      }

      temp[key] = {
        import: `./${mod}.js`,
        require: `./${mod}.cjs`
      }
      return temp
    }),
    // if for some reason in the future we manually add something in the `exports` field
    // this will ensure it doesn't get overwritten by the logic above
    { ...pkg.exports }
  )
}
writeFileSync('dist/package.json', JSON.stringify(outputPkg, undefined, 2))

/**
 * copy necessary files like `CHANGELOG.md` and Licenses to `./dist`
 */
const files = readdirSync('.').filter((f) => f.startsWith('LICENSE'))
files.forEach((f) => copyFileSync(f, `dist/${f}`))

/**
 * copy typescript src files to `./dist`
 */
copySync('src', 'dist')
