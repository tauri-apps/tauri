// rollup.config.js
import { terser } from 'rollup-plugin-terser'
import resolve from '@rollup/plugin-node-resolve'
import commonjs from '@rollup/plugin-commonjs'
import sucrase from '@rollup/plugin-sucrase'
import { getBabelOutputPlugin } from '@rollup/plugin-babel'
import pkg from './package.json'

export default [{
  input: {
    'api/fs': './api-src/fs.ts',
    'api/dialog': './api-src/dialog.ts',
    'api/event': './api-src/event.ts',
    'api/http': './api-src/http.ts',
    'api/index': './api-src/index.ts',
    'api/process': './api-src/process.ts',
    'api/tauri': './api-src/tauri.ts',
    'api/window': './api-src/window.ts',
  },
  treeshake:      true,
  perf:           true,
  output:         [
    {
      dir: 'api/cjs/', // if you want to consume in node but want it tiny
      entryFileNames: '[name].min.js',
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
    'api/bundle': './api-src/bundle.ts'
  },
  output: [{
    name: 'tauri',
    dir:    'api/umd/', // if it needs to run in the browser
    entryFileNames: 'tauri.bundle.min.js',
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
