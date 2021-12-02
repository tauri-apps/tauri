import * as fixtureSetup from '../fixtures/app-test-setup.js'
import { resolve, dirname } from 'path'
import { existsSync, readFileSync, writeFileSync } from 'fs'
import { move } from 'fs-extra'
import { init, build } from 'dist/api/cli'
import { fileURLToPath } from 'url'

const currentDirName = dirname(fileURLToPath(import.meta.url))

describe('[CLI] cli.js template', () => {
  it('init a project and builds it', async () => {
    const cwd = process.cwd()
    const fixturePath = resolve(currentDirName, '../fixtures/empty')
    const tauriFixturePath = resolve(fixturePath, 'src-tauri')
    const outPath = resolve(tauriFixturePath, 'target')
    const cacheOutPath = resolve(fixturePath, 'target')

    fixtureSetup.initJest('empty')

    process.chdir(fixturePath)

    const outExists = existsSync(outPath)
    if (outExists) {
      await move(outPath, cacheOutPath)
    }

    const { promise } = await init({
      directory: process.cwd(),
      force: true,
      tauriPath: resolve(currentDirName, '../../../../..'),
      ci: true
    })
    await promise

    if (outExists) {
      await move(cacheOutPath, outPath)
    }

    process.chdir(tauriFixturePath)

    const manifestPath = resolve(tauriFixturePath, 'Cargo.toml')
    const manifestFile = readFileSync(manifestPath).toString()
    writeFileSync(manifestPath, `workspace = { }\n${manifestFile}`)

    const { promise: buildPromise } = await build()
    await buildPromise
    process.chdir(cwd)
  })
})
