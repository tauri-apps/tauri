// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import execa from 'execa'
import fixtures from 'fixturez'
const f = fixtures(__dirname)
import path from 'path'

const ctaBinary = path.resolve('./bin/create-tauri-app.js')
const clijs = path.resolve('../cli.js/')
const api = path.resolve('../api/')

const manager = process.env.TAURI_RUN_MANAGER ?? 'npm'
const recipes = process.env.TAURI_RECIPE
  ? [process.env.TAURI_RECIPE]
  : ['vanillajs'] //, 'reactjs', 'reactts', 'vite', 'vuecli']
const timeout15m = 900000
const timeout16m = 960000
const logOut = false ? 'inherit' : 'pipe'

beforeAll(async () => {
  const installCLI = await execa('yarn', [], {
    stdio: logOut,
    cwd: clijs,
    timeout: timeout15m
  })

  const buildCLI = await execa('yarn', ['build-release'], {
    stdio: logOut,
    cwd: clijs,
    timeout: timeout15m
  })

  const linkCLI = await execa('yarn', ['link'], {
    stdio: logOut,
    cwd: clijs,
    timeout: timeout15m
  })

  const installAPI = await execa('yarn', [], {
    stdio: logOut,
    cwd: api,
    timeout: timeout15m
  })

  const buildAPI = await execa('yarn', ['build'], {
    stdio: logOut,
    cwd: api,
    timeout: timeout15m
  })

  const linkAPI = await execa('yarn', ['link'], {
    stdio: logOut,
    cwd: api,
    timeout: timeout15m
  })
}, timeout16m)

describe('CTA', () => {
  describe.each(recipes)(`%s recipe`, (recipe) => {
    it(
      'runs',
      async () => {
        const folder = f.temp()

        const child = await execa(
          'node',
          [
            ctaBinary,
            '--manager',
            manager,
            '--recipe',
            recipe,
            '--ci',
            '--dev'
          ],
          {
            all: true,
            stdio: logOut,
            cwd: folder,
            timeout: timeout15m
          }
        )

        expect(child.all).toEqual('')
      },
      timeout16m
    )
  })
})
