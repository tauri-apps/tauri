const path = require('path')
const nodeExternals = require('webpack-node-externals')

module.exports = {
  entry: {
    build: './src/api/build.ts',
    dev: './src/api/dev.ts',
    init: './src/api/init.ts',
    tauricon: './src/api/tauricon.ts',
    'tauri-config': './src/helpers/tauri-config.ts'
  },
  mode: process.env.NODE_ENV || 'development',
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/
      }
    ]
  },
  node: false,
  resolve: {
    extensions: ['.ts', '.js']
  },
  output: {
    library: 'tauri',
    libraryTarget: 'umd',
    filename: '[name].js',
    path: path.resolve(__dirname, 'dist')
  },
  externals: [nodeExternals()],
  target: 'node'
}
