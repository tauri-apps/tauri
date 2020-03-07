jest.setTimeout(240000)
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
      let success = false
      const server = fixtureSetup.startServer(() => {
        success = true
        // wait for the app process to be killed
        setTimeout(resolve, 2000)
      })
      const result = build(tauriConfig)
      await result.promise

      const artifactFolder = tauriConfig.ctx.debug ? 'debug' : 'release'
      const artifactPath = path.resolve(appDir, `src-tauri/target/${artifactFolder}/app`)

      const appPid = spawn(
        process.platform === 'win32' ? `${artifactPath}.exe` : artifactPath.replace(`${artifactFolder}/app`, `${artifactFolder}/./app`),
        [],
        null
      )

      setTimeout(() => {
        if (!success) {
          server.close(() => {
            try {
              process.kill(appPid)
            } catch {}
            reject("App didn't reply")
          })
        }
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

  it.each`
    mode                  | flag
    ${'embedded-server'}  | ${'debug'}
    ${'embedded-server'}  | ${'release'}
    ${'no-server'}        | ${'debug'}
    ${'no-server'}        | ${'release'}
  `('works with the $mode $flag mode', ({ mode, flag }) => {
    return runBuildTest({
      build,
      ctx: {
        debug: flag === 'debug'
      },
      tauri: {
        embeddedServer: {
          active: mode === 'embedded-server'
        }
      }
    })
  })
})
