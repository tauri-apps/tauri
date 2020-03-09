const fixtureSetup = require('../fixtures/app-test-setup')
const { resolve } = require('path')
const { rmdirSync, existsSync, writeFileSync, readFileSync } = require('fs')

describe('[CLI] tauri.js template', () => {
  it('init a project and builds it', async () => {
    const cwd = process.cwd()
    const fixturePath = resolve(__dirname, '../fixtures/empty')
    const tauriFixturePath = resolve(fixturePath, 'src-tauri')

    fixtureSetup.initJest('empty')

    process.chdir(fixturePath)

    if (existsSync(tauriFixturePath)) {
      rmdirSync(tauriFixturePath, { recursive: true })
    }
  
    const { tauri } = require('bin/tauri')
    tauri('init')
    process.chdir(tauriFixturePath)

    const manifestPath = resolve(tauriFixturePath, 'Cargo.toml')
    const manifestFile = readFileSync(manifestPath).toString()
    writeFileSync(manifestPath, `workspace = { }\n\n${manifestFile}`)

    const build = require('api/build')
    await build().promise
    process.chdir(cwd)
  })
})
