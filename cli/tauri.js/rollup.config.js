// rollup.config.js
import { terser } from 'rollup-plugin-terser'
import resolve from '@rollup/plugin-node-resolve'
import commonjs from '@rollup/plugin-commonjs'
import sucrase from '@rollup/plugin-sucrase'
import { getBabelOutputPlugin } from '@rollup/plugin-babel'
import pkg from './package.json'

export default [{
  input: {
    'fs': './api-src/fs.ts',
    'dialog': './api-src/dialog.ts',
    'event': './api-src/event.ts',
    'http': './api-src/http.ts',
    'index': './api-src/index.ts',
    'process': './api-src/process.ts',
    'tauri': './api-src/tauri.ts',
    'window': './api-src/window.ts',
  },
  treeshake:      true,
  perf:           true,
  output:         [
    {
      dir: 'api/', // if you want to consume in node but want it tiny
      entryFileNames: '[name].js',
      format:  'cjs',
      plugins: [ terser() ],
      exports: 'named',
      globals: {}
    },
    {
      dir: 'api/esm/',  // if you will be transpiling and minifying yourself
      entryFileNames: '[name].js',
      format:    'esm',
      sourcemap: true,
      exports: 'named',
      globals: {}
    }
  ],
  plugins: [
    commonjs({}),
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
  ],
  watch:    {
    chokidar: true,
    include: 'api-src/**',
    exclude: 'node_modules/**'
  }
},
{
  input: {
    'bundle': './api-src/bundle.ts'
  },
  output: [{
    name: '__TAURI__',
    dir:    'api/', // if it needs to run in the browser
    entryFileNames: 'tauri.bundle.umd.js',
    format:  'umd',
    plugins: [
      getBabelOutputPlugin({
        presets: [['@babel/preset-env', { modules: 'umd' }]],
        allowAllFormats: true
      }),
      terser()
    ],
    globals: {
    }
  }],
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
}]
