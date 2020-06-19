// rollup.config.js
import { terser } from 'rollup-plugin-terser'
import resolve from '@rollup/plugin-node-resolve'
import commonjs from '@rollup/plugin-commonjs'
import typescript from 'rollup-plugin-typescript2'
import pkg from './package.json'

export default {
  input: {
    'api/fs/index': './api-src/fs/index.ts',
    'api/fs/dir': './api-src/fs/dir.ts',
    'api/dialog': './api-src/dialog.ts',
    'api/event': './api-src/event.ts',
    'api/http': './api-src/http.ts',
    'api/index': './api-src/index.ts',
    'api/process': './api-src/process.ts',
    'api/tauri': './api-src/tauri.ts',
    'api/window': './api-src/window.ts',
  },
  // treeshake:      true,
  // perf:           true,
  output:         [
    {
      dir:    'api-e5/', // if you want to consume in node but want it tiny
      entryFileNames: '[name].cjs.min.js',
      format:  'cjs',
      plugins: [ terser() ],
      exports: 'named',
      globals: {
      }
    },
    {
      dir:      'api-e5/',  // if you will be transpiling and minifying yourself
      entryFileNames: '[name].esm.js',
      format:    'esm',
      sourcemap: true,
      globals:   {
      }
    },
    {
      dir:    'dist/', // if it needs to run in the browser
      entryFileNames: '[name]/[name].umd.min.js',
      format:  'umd',
      plugins: [ terser() ],
      name:    'filer',
      globals: {
      }
    }
  ],
  plugins: [
    commonjs({}),
    typescript({
      typescript: require('typescript'),
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
    include:  'src/**',
    exclude:  'node_modules/**'
  }
}
