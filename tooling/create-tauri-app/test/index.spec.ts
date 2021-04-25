// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import execa from 'execa'
import fixtures from 'fixturez'
const f = fixtures(__dirname)
import path from 'path'

const ctaBinary = path.resolve('./bin/create-tauri-app.js')
const manager = process.env.TAURI_RUN_MANAGER ?? 'npm'
const recipes = process.env.TAURI_RECIPE
  ? [process.env.TAURI_RECIPE]
  : ['vanillajs', 'reactjs', 'reactts', 'vite', 'vuecli']
const timeout15m = 900000
const timeout16m = 960000

describe('CTA', () => {
  describe.each(recipes)(`%s recipe`, (recipe) => {
    it(
      'runs',
      async () => {
        const folder = f.temp()

        const child = await execa(
          'node',
          [ctaBinary, '--manager', manager, '--recipe', recipe, '--ci'],
          {
            all: true,
            stdio: 'pipe',
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
