import * as fixtureSetup from '../fixtures/app-test-setup.js'
import { resolve, dirname } from 'path'
import { writeFileSync, readFileSync } from 'fs'
import { init, build } from 'dist/api/cli'
import { fileURLToPath } from 'url'

const currentDirName = dirname(fileURLToPath(import.meta.url))

describe('[CLI] cli.js template', () => {
  it('init a project and builds it', async () => {
    const cwd = process.cwd()
    const fixturePath = resolve(currentDirName, '../fixtures/empty')
    const tauriFixturePath = resolve(fixturePath, 'src-tauri')

    fixtureSetup.initJest('empty')

    process.chdir(fixturePath)

    const { promise } = await init({
      directory: process.cwd(),
      force: true,
      tauriPath: resolve(currentDirName, '../../../../..'),
      ci: true
    })
    await promise

    process.chdir(tauriFixturePath)

    const manifestPath = resolve(tauriFixturePath, 'Cargo.toml')
    const manifestFile = readFileSync(manifestPath).toString()
    writeFileSync(manifestPath, `workspace = { }\n${manifestFile}`)

    const { promise: buildPromise } = await build()
    await buildPromise
    process.chdir(cwd)
  })
})
