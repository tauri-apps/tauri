// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const path = require('path')
const http = require('http')

const currentDirName = __dirname
const mockFixtureDir = path.resolve(currentDirName, '../fixtures')

module.exports.fixtureDir = mockFixtureDir

module.exports.initJest = (mockFixture) => {
  jest.setTimeout(1200000)

  const mockAppDir = path.join(mockFixtureDir, mockFixture)
  process.env.__TAURI_TEST_APP_DIR = mockAppDir
}

module.exports.startServer = (onSuccess) => {
  const responses = {
    writeFile: null,
    readFile: null,
    writeFileWithDir: null,
    readFileWithDir: null,
    readDir: null,
    readDirWithDir: null,
    copyFile: null,
    copyFileWithDir: null,
    createDir: null,
    createDirWithDir: null,
    removeDir: null,
    removeDirWithDir: null,
    renameFile: null,
    renameFileWithDir: null,
    removeFile: null,
    renameFileWithDir: null,
    listen: null
  }

  function addResponse(response) {
    responses[response.cmd] = true
    if (!Object.values(responses).some((c) => c === null)) {
      server.close(onSuccess)
    }
  }

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
      let body = ''
      req.on('data', (chunk) => {
        body += chunk.toString()
      })
      if (req.url === '/reply') {
        req.on('end', () => {
          const json = JSON.parse(body)
          addResponse(json)
          res.writeHead(200)
          res.end()
        })
      }
      if (req.url === '/error') {
        req.on('end', () => {
          res.writeHead(200)
          res.end()
          throw new Error(body)
        })
      }
    }
  })

  const port = 7000
  const server = app.listen(port)
  return {
    server,
    responses
  }
}
