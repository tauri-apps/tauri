// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { defineConfig, Plugin, RollupLog } from 'rollup'
import typescript from '@rollup/plugin-typescript'
import terser from '@rollup/plugin-terser'
import fg from 'fast-glob'
import { basename, dirname, join } from 'path'
import { copyFileSync, opendirSync, rmSync, Dir } from 'fs'
import { fileURLToPath } from 'url'

// cleanup dist dir
const __dirname = fileURLToPath(new URL('.', import.meta.url))
cleanDir(join(__dirname, './dist'))

const modules = fg.sync(['!./src/*.d.ts', './src/*.ts'])

export default defineConfig([
  {
    input: Object.fromEntries(modules.map((p) => [basename(p, '.ts'), p])),
    output: [
      {
        format: 'esm',
        dir: './dist',
        preserveModules: true,
        preserveModulesRoot: 'src',
        entryFileNames: (chunkInfo) => {
          if (chunkInfo.name.includes('node_modules')) {
            return externalLibPath(chunkInfo.name) + '.js'
          }

          return '[name].js'
        }
      },
      {
        format: 'cjs',
        dir: './dist',
        preserveModules: true,
        preserveModulesRoot: 'src',
        entryFileNames: (chunkInfo) => {
          if (chunkInfo.name.includes('node_modules')) {
            return externalLibPath(chunkInfo.name) + '.cjs'
          }

          return '[name].cjs'
        }
      }
    ],
    plugins: [
      typescript({
        declaration: true,
        declarationDir: './dist',
        rootDir: 'src'
      }),
      makeFlatPackageInDist()
    ],
    onwarn
  },

  {
    input: 'src/index.ts',
    output: {
      format: 'iife',
      name: '__TAURI_IIFE__',
      footer: 'window.__TAURI__ = __TAURI_IIFE__',
      file: '../../crates/tauri/scripts/bundle.global.js'
    },
    plugins: [typescript(), terser()],
    onwarn
  }
])

function externalLibPath(path: string) {
  return `external/${basename(dirname(path))}/${basename(path)}`
}

function onwarn(warning: RollupLog) {
  // deny warnings by default
  throw Object.assign(new Error(), warning)
}

function makeFlatPackageInDist(): Plugin {
  return {
    name: 'makeFlatPackageInDist',
    writeBundle() {
      // copy necessary files like `CHANGELOG.md` , `README.md` and Licenses to `./dist`
      fg.sync('(LICENSE*|*.md|package.json)').forEach((f) =>
        copyFileSync(f, `dist/${f}`)
      )
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
