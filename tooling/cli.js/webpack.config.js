const path = require('path')
const nodeExternals = require('webpack-node-externals')

module.exports = {
  entry: {
    'api/cli': './src/api/cli.ts',
    'api/tauricon': './src/api/tauricon.ts',
    'api/dependency-manager': './src/api/dependency-manager/index.ts',
    'helpers/spawn': './src/helpers/spawn.ts',
    'helpers/rust-cli': './src/helpers/rust-cli.ts',
    'helpers/download-cli': './src/helpers/download-cli.ts'
  },
  mode: process.env.NODE_ENV || 'development',
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/
      },
      {
        test: /(templates|api)[\\/].+\.js/,
        use: 'raw-loader'
      },
      {
        test: /\.toml?$/,
        use: 'toml-loader'
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
    path: path.resolve(__dirname, 'dist'),
    globalObject: 'this'
  },
  externals: [
    nodeExternals({
      allowlist: ['imagemin', 'is-png', 'p-pipe', 'file-type']
    })
  ],
  externalsPresets: { node: true }
}
