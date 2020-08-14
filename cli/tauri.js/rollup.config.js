// rollup.config.js
import { terser } from 'rollup-plugin-terser'
import resolve from '@rollup/plugin-node-resolve'
import commonjs from '@rollup/plugin-commonjs'
import sucrase from '@rollup/plugin-sucrase'
import babel, { getBabelOutputPlugin } from '@rollup/plugin-babel'
import typescript from '@rollup/plugin-typescript'
import pkg from './package.json'

export default [
  {
    input: {
      fs: './api-src/fs.ts',
      dialog: './api-src/dialog.ts',
      event: './api-src/event.ts',
      http: './api-src/http.ts',
      index: './api-src/index.ts',
      process: './api-src/process.ts',
      tauri: './api-src/tauri.ts',
      window: './api-src/window.ts',
      cli: './api-src/cli.ts',
      notification: './api-src/notification.ts'
    },
    treeshake: true,
    perf: true,
    output: [
      {
        dir: 'api/',
        entryFileNames: '[name].js',
        format: 'cjs',
        exports: 'named',
        globals: {}
      },
      {
        dir: 'api/',
        entryFileNames: '[name].mjs',
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
          moduleDirectory: 'node_modules'
        }
      }),
      typescript({
        tsconfig: './tsconfig.api.json'
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
    ],
    watch: {
      chokidar: true,
      include: 'api-src/**',
      exclude: 'node_modules/**'
    }
  },
  {
    input: {
      bundle: './api-src/bundle.ts'
    },
    output: [
      {
        name: '__TAURI__',
        dir: 'api/', // if it needs to run in the browser
        entryFileNames: 'tauri.bundle.umd.js',
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
          moduleDirectory: 'node_modules'
        }
      })
    ],
    external: [
      ...Object.keys(pkg.dependencies || {}),
      ...Object.keys(pkg.peerDependencies || {})
    ]
  }
]
