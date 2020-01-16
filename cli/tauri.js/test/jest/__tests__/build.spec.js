const path = require('path')
const fixtureSetup = require('../fixtures/app-test-setup')
const appDir = fixtureSetup.appDir
const distDir = fixtureSetup.distDir

const spawn = require('../../../src/helpers/spawn').spawn

function runBuildTest(tauriConfig) {
  fixtureSetup.initJest()
  const build = require('../../../src/api/build')
  return new Promise(async (resolve, reject) => {
    try {
      let appPid
      const server = fixtureSetup.startServer(() => appPid, resolve)
      const result = build(tauriConfig)
      await result.promise

      const artifactFolder = tauriConfig.ctx.debug ? 'debug' : 'release'
      const artifactPath = path.resolve(appDir, `src-tauri/target/${artifactFolder}/app`)

      appPid = spawn(
        process.platform === 'win32' ? `${artifactPath}.exe` : artifactPath.replace(`${artifactFolder}/app`, `${artifactFolder}/./app`),
        [],
        null
      )

      setTimeout(() => {
        server.close()
        try {
          process.kill(appPid)
        } catch { }
        reject("App didn't reply")
      }, 2500)
    } catch (error) {
      reject(error)
    }
  })
}

describe('Tauri Build', () => {
  const build = {
    devPath: distDir,
    distDir: distDir
  }

  for (const debug of [true, false]) {
    it(`works with the embedded-server ${debug ? 'debug' : 'release'} mode`, () => {
      return runBuildTest({
        build,
        ctx: {
          debug
        },
        tauri: {
          embeddedServer: {
            active: true
          }
        }
      })
    })
  }

  for (const debug of [true, false]) {
    it(`works with the no-server ${debug ? 'debug' : 'release'} mode`, () => {
      return runBuildTest({
        build,
        ctx: {
          debug
        },
        tauri: {
          embeddedServer: {
            active: false
          }
        }
      })
    })
  }
})
