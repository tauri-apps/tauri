const path = require('path')
const fixtureSetup = require('../fixtures/app-test-setup')
const distDir = fixtureSetup.distDir

function startDevServer() {
    const express = require('express')
    const app = express()

    const port = 7001

    app.get('/', (req, res) => {
        res.sendFile(path.join(distDir, 'index.html'))
    })

    const server = app.listen(port)
    return {
        server,
        url: `http://localhost:${port}`
    }
}

function runDevTest(tauriConfig) {
  fixtureSetup.initJest()
  const dev = require('api/dev')
  return new Promise(async (resolve, reject) => {
    try {
      const { promise, runner } = dev(tauriConfig)
      const server = fixtureSetup.startServer(null, async () => {
        await runner.stop()
        resolve()
      })
      setTimeout(async () => {
        await runner.stop()
        server.close()
        reject("App didn't reply")
      }, 10000)
      await promise
    } catch (error) {
      reject(error)
    }
  })
}

describe('Tauri Dev', () => {
  const build = {
    distDir: distDir
  }

  const devServer = startDevServer()

  for (const devPath of [devServer.url, distDir]) {
    const runningDevServer = devPath.startsWith('http')
    it(`works with the ${runningDevServer ? 'dev server' : 'dev path'} mode`, () => {
      const promise = runDevTest({
        build: {
          ...build,
          devPath
        },
        ctx: {
          debug: true,
          dev: true
        }
      })

      promise.then(() => {
        if (runningDevServer) {
          devServer.server.close()
        }
      })

      return promise
    })
  }
})
