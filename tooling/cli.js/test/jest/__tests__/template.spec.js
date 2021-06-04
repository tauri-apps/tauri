const fixtureSetup = require('../fixtures/app-test-setup')
const { resolve } = require('path')
const { writeFileSync, readFileSync } = require('fs')

describe('[CLI] cli.js template', () => {
  it('init a project and builds it', async () => {
    const cwd = process.cwd()
    const fixturePath = resolve(__dirname, '../fixtures/empty')
    const tauriFixturePath = resolve(fixturePath, 'src-tauri')

    fixtureSetup.initJest('empty')

    process.chdir(fixturePath)

    const { init, build } = require('dist/api/cli')
    const { promise } = await init({
      directory: process.cwd(),
      force: true,
      tauriPath: resolve(__dirname, '../../../../..'),
      ci: true
    })
    await promise

    process.chdir(tauriFixturePath)

    const manifestPath = resolve(tauriFixturePath, 'Cargo.toml')
    const manifestFile = readFileSync(manifestPath).toString()
    writeFileSync(
      manifestPath,
      `workspace = { }\n[patch.crates-io]\ntao = { git = "https://github.com/tauri-apps/tao", rev = "5be88eb9488e3ad27194b5eff2ea31a473128f9c" }\n\n${manifestFile}`
    )

    const { promise: buildPromise } = await build()
    await buildPromise
    process.chdir(cwd)
  })
})
