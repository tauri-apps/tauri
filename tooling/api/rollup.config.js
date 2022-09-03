// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// rollup.config.js
import { readdirSync } from 'fs'
import { terser } from 'rollup-plugin-terser'
import resolve from '@rollup/plugin-node-resolve'
import commonjs from '@rollup/plugin-commonjs'
import sucrase from '@rollup/plugin-sucrase'
import babel, { getBabelOutputPlugin } from '@rollup/plugin-babel'
import typescript from '@rollup/plugin-typescript'
import pkg from './package.json'

export default [
  {
    input: (() => {
      let input = {}
      readdirSync('src')
        .filter((e) => e.endsWith('.ts') && e !== 'bundle.ts')
        .forEach((mod) => (input[`${mod.replace('.ts', '')}`] = `./src/${mod}`))
      return input
    })(),
    treeshake: true,
    perf: true,
    output: [
      {
        dir: 'dist/',
        entryFileNames: '[name].cjs',
        format: 'cjs',
        chunkFileNames: '[name]-[hash].cjs',
        exports: 'named',
        globals: {}
      },
      {
        dir: 'dist/',
        entryFileNames: '[name].js',
        format: 'esm',
        exports: 'named',
        globals: {}
      }
    ],
    plugins: [
      commonjs({}),
      resolve({
        // pass custom options to the resolve plugin
        customResolveOptions: {
          moduleDirectories: ['node_modules']
        }
      }),
      typescript({
        tsconfig: './tsconfig.json'
      }),
      babel({
        configFile: false,
        presets: [['@babel/preset-env'], ['@babel/preset-typescript']],
        babelHelpers: 'bundled'
      }),
      terser()
    ],
    external: [
      ...Object.keys(pkg.dependencies || {}),
      ...Object.keys(pkg.peerDependencies || {})
    ]
  },
  {
    input: {
      bundle: './src/bundle.ts'
    },
    output: [
      {
        name: '__TAURI__',
        dir: '../../core/tauri/scripts',
        entryFileNames: 'bundle.js',
        format: 'umd',
        plugins: [
          getBabelOutputPlugin({
            presets: [['@babel/preset-env', { modules: 'umd' }]],
            allowAllFormats: true
          }),
          terser()
        ],
        globals: {}
      }
    ],
    plugins: [
      sucrase({
        exclude: ['node_modules'],
        transforms: ['typescript']
      }),
      resolve({
        // pass custom options to the resolve plugin
        customResolveOptions: {
          moduleDirectories: ['node_modules']
        }
      })
    ],
    external: [
      ...Object.keys(pkg.dependencies || {}),
      ...Object.keys(pkg.peerDependencies || {})
    ]
  }
]
