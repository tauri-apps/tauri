// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { main, spawn, sleep } from 'effection'
import { exec } from '@effection/process'
import fs from 'fs/promises'
import path from 'path'
import crypto from 'crypto'
import tempDirectory from 'temp-dir'

let assert
const nodeVersion = process.versions.node.split('.')[0]
if (nodeVersion === '14') {
  assert = await import('assert')
} else {
  assert = await import('assert/strict')
}

const ctaBinary = path.resolve('./bin/create-tauri-app.js')
const clijs = path.resolve('../cli.js/')
const api = path.resolve('../api/')

const manager = process.env.TAURI_RUN_MANAGER || 'yarn'
const recipes = process.env.TAURI_RECIPE
  ? process.env.TAURI_RECIPE.split(',')
  : ['vanillajs', 'cra', 'vite', 'vuecli', 'ngcli']
const parallelize = process.env.TAURI_RECIPE_PARALLELIZE || false

main(function* start() {
  const tauriTemp = path.join(
    tempDirectory,
    `tauri_${crypto.randomBytes(16).toString('hex')}`
  )

  try {
    const appName = 'tauri-app'
    let output = {}
    for (let i = 0; i < recipes.length; i++) {
      const recipe = recipes[i]
      console.log(`::group::recipe ${recipe}`)
      console.log(`------------------ ${recipe} started -------------------`)
      const recipeFolder = path.join(tauriTemp, recipe)
      const appFolder = path.join(recipeFolder, appName)
      yield fs.mkdir(recipeFolder, { recursive: true })
      console.log(`${recipeFolder} created.`)

      // runs CTA with all args set to avoid any prompts
      const run = `node '${ctaBinary}' --manager ${manager} --recipe ${recipe} --ci --dev`
      console.log(`[running] ${run}`)

      const cta = yield exec(run, { cwd: recipeFolder })
      if (!parallelize) yield streamLogs(cta)
      else {
        console.log('running CTA recipe in parallel')
        output[recipe] = { cta: yield spawn(cta.join()) }
      }

      // now it is finished, assert on some things
      yield assertCTAState({ appFolder, appName })

      let opts = []
      if (manager === 'npm') {
        opts =
          recipe == 'vuecli'
            ? ['run', 'tauri:build']
            : ['run', 'tauri', '--', 'build']
      } else if (manager === 'yarn') {
        opts = recipe == 'vuecli' ? ['tauri:build'] : ['tauri', 'build']
      }

      const tauriBuild = yield exec(manager, {
        arguments: opts,
        cwd: appFolder
      })
      if (!parallelize) yield streamLogs(tauriBuild)
      else {
        console.log('running recipe tauri build in parallel')
        output[recipe].tauriBuild = yield spawn(tauriBuild.join())
      }

      // build is complete, assert on some things
      yield assertTauriBuildState({ appFolder, appName })

      console.log(`------------------ ${recipe} complete -------------------`)
      console.log('::endgroup::')
      // sometimes it takes a moment to flush all of the logs
      // to the console, let things catch up here
      yield sleep(1000)
    }

    if (parallelize) {
      for (let i = 0; i < recipes.length; i++) {
        const recipe = recipes[i]
        console.log(`::group::recipe ${recipe} logs`)
        console.log(
          `------------------ ${recipe} output start -------------------`
        )
        const ctaOutput = yield output[recipe].cta
        console.log(ctaOutput)
        yield output[recipe].tauriBuild
        console.log(
          `------------------ ${recipe} output end -------------------`
        )
        console.log('::endgroup::')
      }
    }
  } catch (e) {
    console.error(e)
    throw Error(e)
  } finally {
    console.log('\nstopping process...')
    // wait a tick for file locks to be release
    yield sleep(5000)
    yield fs.rm(tauriTemp, { recursive: true, force: true })
    console.log(`${tauriTemp} deleted.`)
  }
})

function* streamLogs(child) {
  yield child.stdout.forEach((data) => {
    process.stdout.write(data)
  })
  yield child.stderr.forEach((data) => {
    process.stderr.write(data)
  })
}

function* assertCTAState({ appFolder, appName }) {
  const packageFileInitial = JSON.parse(
    yield fs.readFile(path.join(appFolder, 'package.json'), 'utf-8')
  )
  assert.strictEqual(
    packageFileInitial.name,
    appName,
    `The package.json did not have the name "${appName}".`
  )
  assert.strictEqual(
    packageFileInitial.scripts.tauri,
    'tauri',
    `The package.json did not have the tauri script.`
  )
}

function* assertTauriBuildState({ appFolder, appName }) {
  const packageFileOutput = JSON.parse(
    yield fs.readFile(path.join(appFolder, 'package.json'), 'utf-8')
  )
  assert.strictEqual(
    packageFileOutput.name,
    appName,
    `The package.json did not have the name "${appName}".`
  )
  assert.strictEqual(
    packageFileOutput.scripts.tauri,
    'tauri',
    `The package.json did not have the tauri script.`
  )

  const cargoFileOutput = yield fs.readFile(
    path.join(appFolder, 'src-tauri', 'Cargo.toml'),
    'utf-8'
  )
  assert.strictEqual(
    cargoFileOutput.startsWith(`[package]\nname = "app"`),
    true,
    `The Cargo.toml did not have the name "app".`
  )

  const tauriTarget = yield fs.readdir(
    path.join(appFolder, 'src-tauri', 'target')
  )
  assert.strictEqual(
    tauriTarget.includes('release'),
    true,
    `The Tauri build does not have a target/release directory.`
  )
}
