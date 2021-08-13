const path = require('path')
const nodeExternals = require('webpack-node-externals')

module.exports = {
  target: 'es2020',
  entry: {
    'api/cli': './src/api/cli.ts',
    'api/tauricon': './src/api/tauricon.ts',
    'api/dependency-manager': './src/api/dependency-manager/index.ts',
    'helpers/spawn': './src/helpers/spawn.ts',
    'helpers/rust-cli': './src/helpers/rust-cli.ts',
    'helpers/download-binary': './src/helpers/download-binary.ts'
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
    library: {
      type: 'module'
    },
    filename: '[name].js',
    path: path.resolve(__dirname, 'dist'),
    globalObject: 'this'
  },
  experiments: {
    outputModule: true
  },
  externals: [
    nodeExternals({
      importType: 'module'
    })
  ],
  externalsPresets: { node: true }
}
