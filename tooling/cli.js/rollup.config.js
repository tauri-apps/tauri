// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// rollup.config.js
import { readFileSync } from 'fs'
import { terser } from 'rollup-plugin-terser'
import resolve from '@rollup/plugin-node-resolve'
import commonjs from '@rollup/plugin-commonjs'
import babel from '@rollup/plugin-babel'
import typescript from '@rollup/plugin-typescript'
import pkg from './package.json'
import replace from '@rollup/plugin-replace'
import TOML from '@tauri-apps/toml'

const cliManifestContents = readFileSync('../cli.rs/Cargo.toml').toString()
const cliManifest = TOML.parse(cliManifestContents)

export default {
  input: {
    'api/cli': './src/api/cli.ts',
    'api/tauricon': './src/api/tauricon.ts',
    'api/dependency-manager': './src/api/dependency-manager/index.ts',
    'helpers/spawn': './src/helpers/spawn.ts',
    'helpers/rust-cli': './src/helpers/rust-cli.ts',
    'helpers/download-binary': './src/helpers/download-binary.ts'
  },
  treeshake: true,
  perf: true,
  output: {
    dir: 'dist/',
    entryFileNames: '[name].js',
    format: 'esm',
    exports: 'named',
    globals: {}
  },
  plugins: [
    replace({
      __RUST_CLI_VERSION__: JSON.stringify(cliManifest.package.version),
      'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV)
    }),
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
      presets: [['@babel/preset-env'], ['@babel/preset-typescript']]
    }),
    terser()
  ],
  external: [
    ...Object.keys(pkg.dependencies || {}),
    ...Object.keys(pkg.peerDependencies || {})
  ]
}
