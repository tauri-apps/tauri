const path = require('path')
const process = require('process')

const appDir = path.resolve(__dirname, '../fixtures/app')
const tauriDir = path.join(appDir, 'src-tauri')

module.exports.appDir = appDir
module.exports.distDir = path.join(appDir, 'dist')

import * as appPaths from '../../../src/helpers/app-paths'

module.exports.initJest = () => {
  jest.mock('helpers/non-webpack-require', () => {
    return path => {
      const value = require('fs').readFileSync(path).toString()
      if (path.endsWith('.json')) {
        return JSON.parse(value)
      }
      return value
    }
  })
  appPaths.appDir = appDir
  appPaths.tauriDir = tauriDir
  jest.spyOn(appPaths.resolve, 'app').mockImplementation(dir => path.resolve(appDir, dir))
  jest.spyOn(appPaths.resolve, 'tauri').mockImplementation(dir => path.resolve(tauriDir, dir))
  jest.spyOn(process, 'exit').mockImplementation(() => {})
}

module.exports.startServer = (onReply) => {
  const http = require('http')
  const app = http.createServer((req, res) => {
    // Set CORS headers
    res.setHeader('Access-Control-Allow-Origin', '*')
    res.setHeader('Access-Control-Request-Method', '*')
    res.setHeader('Access-Control-Allow-Methods', 'OPTIONS, GET')
    res.setHeader('Access-Control-Allow-Headers', '*')

    if (req.method === 'OPTIONS') {
      res.writeHead(200)
      res.end()
      return
    }

    if (req.method === 'POST') {
      if (req.url === '/reply') {
        let body = ''
        req.on('data', chunk => {
          body += chunk.toString()
        })
        req.on('end', () => {
          expect(JSON.parse(body)).toStrictEqual({
            msg: 'TEST'
          })
          server.close()
          onReply()
        })
      }
    }
  })

  const port = 7000
  const server = app.listen(port)
  return server
}
