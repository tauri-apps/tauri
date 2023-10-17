// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { defineConfig, RollupOptions, Plugin } from 'rollup'
import typescript from '@rollup/plugin-typescript'
import terser from '@rollup/plugin-terser'
import { dts } from 'rollup-plugin-dts'
import fg from 'fast-glob'
import { basename, join } from 'path'
import {
  readFileSync,
  readdirSync,
  writeFileSync,
  copyFileSync,
  opendirSync,
  rmSync,
  Dir
} from 'fs'
import { fileURLToPath } from 'url'

// cleanup dist dir
const __dirname = fileURLToPath(new URL('.', import.meta.url))
cleanDir(join(__dirname, './dist'))

const mainConfig: RollupOptions = {
  input: Object.fromEntries(
    fg.sync('./src/*.ts').map((p) => [basename(p, '.ts'), p])
  ),
  output: {
    format: 'esm',
    dir: './dist',
    preserveModules: true
  },
  plugins: [typescript(), makeFlatPackageInDist()]
}

export default defineConfig([
  mainConfig,
  {
    ...mainConfig,
    plugins: [typescript(), dts()]
  },
  {
    input: 'src/index.ts',
    output: {
      format: 'iife',
      name: '__TAURI__',
      file: '../../core/tauri/scripts/bundle.js'
    },
    plugins: [typescript(), terser()]
  }
])

function makeFlatPackageInDist(): Plugin {
  return {
    name: 'makeFlatPackageInDist',
    writeBundle() {
      // append our api modules to `exports` in `package.json` then write it to `./dist`
      const pkg = JSON.parse(readFileSync('package.json', 'utf8'))
      const modules = readdirSync('src')
        .filter((e) => e !== 'helpers')
        .map((mod) => mod.replace('.ts', ''))

      const outputPkg = {
        ...pkg,
        devDependencies: {},
        exports: Object.assign(
          {},
          ...modules.map((mod) => {
            let temp: Record<string, { import: string; require: string }> = {}
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
          { ...(pkg.exports || {}) }
        )
      }
      writeFileSync(
        'dist/package.json',
        JSON.stringify(outputPkg, undefined, 2)
      )

      // copy necessary files like `CHANGELOG.md` , `README.md` and Licenses to `./dist`
      const dir = readdirSync('.')
      const files = [
        ...dir.filter((f) => f.startsWith('LICENSE')),
        ...dir.filter((f) => f.endsWith('.md'))
      ]
      files.forEach((f) => copyFileSync(f, `dist/${f}`))
    }
  }
}

function cleanDir(path: string) {
  let dir: Dir
  try {
    dir = opendirSync(path)
  } catch (err: any) {
    switch (err.code) {
      case 'ENOENT':
        return // Noop when directory don't exists.
      case 'ENOTDIR':
        throw new Error(`'${path}' is not a directory.`)
      default:
        throw err
    }
  }

  let file = dir.readSync()
  while (file) {
    const filePath = join(path, file.name)
    rmSync(filePath, { recursive: true })
    file = dir.readSync()
  }
  dir.closeSync()
}
