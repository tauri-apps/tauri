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
}

module.exports.startServer = (getAppPid, onReply) => {
  const express = require('express')
  const cors = require('cors')
  const app = express()

  app.use(cors())
  app.use(express.json())

  const port = 7000

  app.post('/reply', (req, res) => {
    expect(req.body).toStrictEqual({
      msg: 'TEST'
    })
    server.close()
    if (typeof getAppPid === 'function') {
      process.kill(getAppPid())
    }
    // wait for the app process to be killed
    setTimeout(() => {
      onReply()
    }, 100)
  })

  const server = app.listen(port)
}
