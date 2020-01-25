const path = require('path')
const fixtureSetup = require('../fixtures/app-test-setup')
const distDir = fixtureSetup.distDir

function startDevServer() {
    const http = require('http')
    const { statSync, createReadStream } = require('fs')
    const app = http.createServer((req, res) => {
      if (req.method === 'GET') {
        if (req.url === '/') {
          const indexPath = path.join(distDir, 'index.html')
          const stat = statSync(indexPath)
          res.writeHead(200, {
            'Content-Type': 'text/html',
            'Content-Length': stat.size
          })
          createReadStream(indexPath).pipe(res)
        }
      }
    })

    const port = 7001

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

  test.each([[devServer.url], [distDir]])(
    'works with dev pointing to $a',
    devPath => {
      const runningDevServer = devPath.startsWith('http')
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
    }
  )
})
