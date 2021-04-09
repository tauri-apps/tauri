const path = require('path')
const process = require('process')

const mockFixtureDir = path.resolve(__dirname, '../fixtures')

module.exports.fixtureDir = mockFixtureDir

function mockResolvePath(basePath, dir) {
  return dir && path.isAbsolute(dir) ? dir : path.resolve(basePath, dir)
}

module.exports.initJest = (mockFixture) => {
  jest.setTimeout(1200000)
  jest.mock('helpers/non-webpack-require', () => {
    return (path) => {
      const value = require('fs').readFileSync(path).toString()
      if (path.endsWith('.json')) {
        return JSON.parse(value)
      }
      return value
    }
  })

  jest.mock('helpers/app-paths', () => {
    const path = require('path')
    const appDir = path.join(mockFixtureDir, mockFixture)
    const tauriDir = path.join(appDir, 'src-tauri')
    return {
      appDir,
      tauriDir,
      resolve: {
        app: (dir) => mockResolvePath(appDir, dir),
        tauri: (dir) => mockResolvePath(tauriDir, dir)
      }
    }
  })
}

module.exports.startServer = (onSuccess) => {
  const http = require('http')

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
