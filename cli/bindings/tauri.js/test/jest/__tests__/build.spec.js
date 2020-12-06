const path = require('path')
const fixtureSetup = require('../fixtures/app-test-setup')
const appDir = path.join(fixtureSetup.fixtureDir, 'app')
const distDir = path.join(appDir, 'dist')

const spawn = require('helpers/spawn').spawn

function runBuildTest(tauriConfig) {
  fixtureSetup.initJest('app')
  const build = require('api/build')
  return new Promise(async (resolve, reject) => {
    try {
      let success = false
      const { server, responses } = fixtureSetup.startServer(() => {
        success = true
        try {
          process.kill(appPid)
        } catch {}
        // wait for the app process to be killed
        setTimeout(resolve, 2000)
      })
      const result = build(tauriConfig)
      await result.promise

      const artifactFolder = tauriConfig.ctx.debug ? 'debug' : 'release'
      const artifactPath = path.resolve(
        appDir,
        `src-tauri/target/${artifactFolder}/app`
      )

      const appPid = spawn(
        process.platform === 'win32'
          ? `${artifactPath}.exe`
          : artifactPath.replace(
              `${artifactFolder}/app`,
              `${artifactFolder}/./app`
            ),
        [],
        null
      )

      setTimeout(() => {
        if (!success) {
          server.close(() => {
            try {
              process.kill(appPid)
            } catch {}
            const failedCommands = Object.keys(responses)
              .filter((k) => responses[k] === null)
              .join(', ')
            reject("App didn't reply to " + failedCommands)
          })
        }
      }, 15000)
    } catch (error) {
      reject(error)
    }
  })
}

describe('Tauri Build', () => {
  const build = {
    devPath: distDir,
    distDir: distDir,
    withGlobalTauri: true
  }

  it.each`
    mode                 | flag
    ${'embedded-server'} | ${'debug'}
    ${'embedded-server'} | ${'release'}
    ${'no-server'}       | ${'debug'}
    ${'no-server'}       | ${'release'}
  `('works with the $mode $flag mode', ({ mode, flag }) => {
    return runBuildTest({
      build,
      ctx: {
        debug: flag === 'debug'
      },
      tauri: {
        allowlist: {
          all: true
        },
        embeddedServer: {
          active: mode === 'embedded-server'
        }
      }
    })
  })
})
