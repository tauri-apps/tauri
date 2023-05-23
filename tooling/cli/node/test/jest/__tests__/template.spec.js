// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const fixtureSetup = require('../fixtures/app-test-setup.js')
const { resolve } = require('path')
const { existsSync, readFileSync, writeFileSync } = require('fs')
const { move } = require('fs-extra')
const cli = require('~/main.js')

const currentDirName = __dirname

describe('[CLI] @tauri-apps/cli template', () => {
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

    await cli.run(['init', '--directory', process.cwd(), '--force', '--tauri-path', resolve(currentDirName, '../../../../../..'), '--ci'])

    if (outExists) {
      await move(cacheOutPath, outPath)
    }

    process.chdir(tauriFixturePath)

    const manifestPath = resolve(tauriFixturePath, 'Cargo.toml')
    const manifestFile = readFileSync(manifestPath).toString()
    writeFileSync(manifestPath, `workspace = { }\n${manifestFile}`)

    const configPath = resolve(tauriFixturePath, 'tauri.conf.json')
    const config = readFileSync(configPath).toString()
    writeFileSync(configPath, config.replace('com.tauri.dev', 'com.tauri.test'))

    await cli.run(['build', '--verbose'])
    process.chdir(cwd)
  })
})
