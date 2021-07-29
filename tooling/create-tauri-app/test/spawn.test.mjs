// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { main, spawn, sleep } from 'effection'
import { exec } from '@effection/process'
import fs from 'fs/promises'
import path from 'path'
import crypto from 'crypto'
import tempDirectory from 'temp-dir'

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

      let opts = []
      if (manager === 'npm') {
        opts =
          recipe == 'vuecli'
            ? ['run', 'tauri:build']
            : ['run', 'tauri', '--', 'build']
      } else if (manager === 'yarn') {
        opts = recipe == 'vuecli' ? ['tauri:build'] : ['tauri', 'build']
      }

      // now it is finished, assert on some things
      yield assertCTAState()

      const tauriBuild = yield exec(manager, {
        arguments: opts,
        cwd: recipeFolder
      })
      if (!parallelize) yield streamLogs(tauriBuild)
      else {
        console.log('running recipe tauri build in parallel')
        output[recipe].tauriBuild = yield spawn(tauriBuild.join())
      }

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

    // build is complete, assert on some things
    yield assertTauriBuildState()
  } catch (e) {
    console.error(e)
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

function* assertCTAState() {
  console.log('TODO: assert on state of fs after CTA runs')
}

function* assertTauriBuildState() {
  console.log('TODO: assert on state of fs after Tauri Build runs')
}
