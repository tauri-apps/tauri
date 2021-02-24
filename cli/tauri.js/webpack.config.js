const path = require('path')
const nodeExternals = require('webpack-node-externals')

module.exports = {
  entry: {
    'api/cli': './src/api/cli.ts',
    'api/init': './src/api/init.ts',
    'api/tauricon': './src/api/tauricon.ts',
    'api/info': './src/api/info.ts',
    'api/dependency-manager': './src/api/dependency-manager/index.ts',
    'helpers/spawn': './src/helpers/spawn.ts',
    'helpers/rust-cli': './src/helpers/rust-cli.ts'
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
    path: path.resolve(__dirname, 'dist')
  },
  externals: [nodeExternals()],
  target: 'node'
}

function schemaParser(schemaName, content) {
  const lines = content.split('\n')
  const output = []

  for (const line of lines) {
    if (line === `export const ${schemaName} = {`) {
      output.push('{')
    } else if (output.length) {
      if (line === '}') {
        output.push('}')
        break
      }
      output.push(line)
    }
  }

  const json = output.join('\n')
  const object = eval(`(${json})`)
  return JSON.stringify(object, null, 2)
}
