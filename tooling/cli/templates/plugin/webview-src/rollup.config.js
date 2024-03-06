import { nodeResolve } from '@rollup/plugin-node-resolve'
import typescript from '@rollup/plugin-typescript'
import  terser from '@rollup/plugin-terser'

export default {
  input: './webview-src/index.ts',
  output: {
    dir: './webview-dist',
    entryFileNames: '[name].js',
    format: 'es',
    exports: 'auto'
  },
  plugins: [
    nodeResolve(),
    terser(),
    typescript({
      tsconfig: './webview-src/tsconfig.json',
      moduleResolution: 'node'
    })
  ]
}
