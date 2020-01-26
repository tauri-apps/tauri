const path = require('path')
const fixtureSetup = require('../fixtures/app-test-setup')
const appDir = fixtureSetup.appDir
const distDir = fixtureSetup.distDir

const spawn = require('helpers/spawn').spawn

function runBuildTest(tauriConfig) {
  fixtureSetup.initJest()
  const build = require('api/build')
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

  it.each([['debug'], ['release']])(
    'works with the embedded-server $a mode',
    mode => {
      return runBuildTest({
        build,
        ctx: {
          debug: mode === 'debug'
        },
        tauri: {
          embeddedServer: {
            active: true
          }
        }
      })
    }
  )

  it.each([['debug'], ['release']])(
    'works with the no-server $a mode',
    mode => {
      return runBuildTest({
        build,
        ctx: {
          debug: mode === 'debug'
        },
        tauri: {
          embeddedServer: {
            active: false
          }
        }
      })
    }
  )
})
