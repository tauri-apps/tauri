const express = require('express')
const cors = require('cors')
const app = express()
const path = require('path')

const appDir = path.resolve(__dirname, '../fixtures/app')
const distDir = path.join(appDir, 'dist')
const spy = jest.spyOn(process, 'cwd')
spy.mockReturnValue(appDir)

const spawn = require('../../../dist/helpers/spawn').spawn
const build = require('../../../dist/api/build')
app.use(cors())
app.use(express.json())

describe('it builds a Tauri app', () => {
  test('it pings a localhost server', () => {
    return new Promise(async (resolve, reject) => {
      const port = 7000
      let appPid

      app.post('/reply', (req, res) => {
        expect(req.body).toStrictEqual({
          msg: 'TEST'
        })
        console.log(req.body)
        server.close()
        process.kill(appPid)
        // wait for the app process to be killed
        setTimeout(() => {
          resolve()
        })
      })

      const server = app.listen(port, () => console.log(`Test listening on port ${port}!`))

      try {
        await build({
          build: {
            devPath: distDir,
            distDir: distDir
          },
          ctx: {
            debug: true
          },
          tauri: {
            embeddedServer: {
              active: true
            }
          }
        })

        const artifactPath = path.resolve(appDir, 'src-tauri/target/debug/app')

        appPid = spawn(
          process.platform === 'win32' ? `${artifactPath}.exe` : artifactPath.replace('debug/app', 'debug/./app'),
          [],
          null
        )

        setTimeout(() => {
          reject("App didn't reply")
        }, 2500)
      } catch (error) {
        reject(error)
      }
    })
  })
})
